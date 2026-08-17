package hara.truffle;

import hara.lang.protocol.IApplicable;
import hara.lang.protocol.IComponent;
import hara.lang.protocol.IContext;
import hara.lang.protocol.IInvokeIn;
import hara.lang.protocol.IMetadata;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collections;
import java.util.List;
import java.util.Set;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.concurrent.atomic.AtomicLong;
import java.util.concurrent.atomic.AtomicReference;
import java.util.function.Consumer;
import org.graalvm.polyglot.Context;
import org.graalvm.polyglot.HostAccess;
import org.graalvm.polyglot.PolyglotException;
import org.graalvm.polyglot.Source;
import org.graalvm.polyglot.Value;
import org.graalvm.polyglot.io.IOAccess;

/** Owns the runtime contexts shared by local and RESP clients. */
final class SessionKernel implements AutoCloseable {
  private static final ConcurrentHashMap<String, SessionKernel> EMBEDDINGS =
      new ConcurrentHashMap<>();

  static SessionKernel embedding(String token) {
    return token == null || token.isEmpty() ? null : EMBEDDINGS.get(token);
  }
  /**
   * Host authority applied when a context is created.
   *
   * <p>This policy does not include a filesystem mounted explicitly through {@link
   * SessionKernel#attachFilesystem(SessionModel.SessionId, SessionModel.SessionMountId)}. Such a
   * mount is a separately delegated, scoped resource. In-process namespace and context separation
   * remains logical isolation, not a security boundary.
   */
  static final class SessionAuthorityPolicy {
    static final SessionAuthorityPolicy ZERO =
        new SessionAuthorityPolicy(false, false, false, false, false, false);

    final boolean hostFilesystem;
    final boolean hostNetwork;
    final boolean hostProcess;
    final boolean reflection;
    final boolean packages;
    final boolean project;

    SessionAuthorityPolicy(
        boolean hostFilesystem,
        boolean hostNetwork,
        boolean hostProcess,
        boolean reflection,
        boolean packages,
        boolean project) {
      this.hostFilesystem = hostFilesystem;
      this.hostNetwork = hostNetwork;
      this.hostProcess = hostProcess;
      this.reflection = reflection;
      this.packages = packages;
      this.project = project;
    }

    static SessionAuthorityPolicy root(
        boolean allowFile, boolean allowNetwork, boolean allowProcess, HaraProject project) {
      return new SessionAuthorityPolicy(
          allowFile,
          allowNetwork,
          allowProcess,
          project != null && project.hasCapability("jvm/reflection"),
          project != null,
          project != null);
    }

    String profile() {
      return hostFilesystem || hostNetwork || hostProcess || reflection || packages || project
          ? "explicit"
          : "zero";
    }
  }

  private final boolean allowFile;
  private final String embeddingToken = java.util.UUID.randomUUID().toString();
  private final SessionRegistry sessionRegistry = new SessionRegistry();
  private final MountRegistry mountRegistry = new MountRegistry();
  private final DevelopmentResourceCatalog developmentResources =
      new DevelopmentResourceCatalog();
  private final BundleCatalog bundleCatalog = new BundleCatalog();
  private final SandboxProviderRegistry sandboxProviderRegistry =
      new SandboxProviderRegistry();
  private final SandboxRegistry sandboxRegistry = new SandboxRegistry();
  private static final SessionModel.SessionId ROOT_ID = SessionModel.SessionId.parse("ROOT");

  private static final class SessionRegistry {
    final ConcurrentHashMap<String, Session> entries = new ConcurrentHashMap<>();
  }

  private static final class MountRegistry {
    final ConcurrentHashMap<Long, FilesystemMount> entries = new ConcurrentHashMap<>();
    final ConcurrentHashMap<String, Long> sessionAttachments = new ConcurrentHashMap<>();
    final ConcurrentHashMap<Long, Long> sandboxAttachments = new ConcurrentHashMap<>();
    final AtomicLong nextId = new AtomicLong(1);
  }

  private static final class DevelopmentResourceCatalog {
    final ConcurrentHashMap<String, String> entries = new ConcurrentHashMap<>();
  }

  private static final class BundleCatalog {
    final ConcurrentHashMap<String, byte[]> entries = new ConcurrentHashMap<>();
  }

  private static final class SandboxProviderRegistry {
    final ConcurrentHashMap<String, SandboxProvider> entries = new ConcurrentHashMap<>();
  }

  private static final class SandboxRegistry {
    final ConcurrentHashMap<Long, Sandbox> entries = new ConcurrentHashMap<>();
    final AtomicLong nextId = new AtomicLong(1);
  }

  private static final class FilesystemMount {
    final HaraMountedFileSystem provider;
    final Path root;
    int attachments;

    FilesystemMount(HaraMountedFileSystem provider, Path root) {
      this.provider = provider;
      this.root = root;
    }
  }

  record FilesystemInfo(String kind, Path root, int attachments) {}

  SessionKernel(boolean allowFile, boolean allowNetwork) {
    this(allowFile, allowNetwork, false);
  }

  SessionKernel(boolean allowFile, boolean allowNetwork, boolean allowProcess) {
    this(allowFile, allowNetwork, allowProcess, null);
  }

  SessionKernel(
      boolean allowFile, boolean allowNetwork, boolean allowProcess, HaraProject project) {
    this.allowFile = allowFile;
    EMBEDDINGS.put(embeddingToken, this);
    registerSandboxProvider(InProcessSandboxProvider.INSTANCE);
    SessionAuthorityPolicy rootAuthority =
        SessionAuthorityPolicy.root(allowFile, allowNetwork, allowProcess, project);
    sessionRegistry.entries.put(
        ROOT_ID.value(),
        new Session(
            new SessionModel.SessionSpec(ROOT_ID, rootAuthority),
            project,
            mount -> releaseMount(ROOT_ID, mount),
            false,
            embeddingToken));
  }

  Session root() {
    return require(ROOT_ID);
  }

  Session require(SessionModel.SessionId id) {
    Session session = sessionRegistry.entries.get(id.value());
    if (session == null) throw new IllegalArgumentException("NO_SESSION " + id);
    return session;
  }

  synchronized Session create(SessionModel.SessionId id) {
    if (sessionRegistry.entries.containsKey(id.value()))
      throw new IllegalArgumentException("SESSION_EXISTS " + id);
    Session session =
        new Session(
            SessionModel.SessionSpec.zeroAuthority(id),
            null,
            mount -> releaseMount(id, mount),
            false,
            embeddingToken);
    sessionRegistry.entries.put(id.value(), session);
    return session;
  }

  synchronized SessionModel.SessionMountId createFilesystem(Path root) {
    if (!allowFile) throw new IllegalArgumentException("FILE_ACCESS_DENIED");
    Path normalized = root.toAbsolutePath().normalize();
    if (!Files.isDirectory(normalized)) {
      throw new IllegalArgumentException("FILESYSTEM_NOT_FOUND " + normalized);
    }
    long value = mountRegistry.nextId.getAndIncrement();
    if (value <= 0) throw new IllegalStateException("FILESYSTEM_IDS_EXHAUSTED");
    SessionModel.SessionMountId id = SessionModel.SessionMountId.of(value);
    mountRegistry.entries.put(
        value, new FilesystemMount(new HaraMountedFileSystem(normalized), normalized));
    return id;
  }

  synchronized void attachFilesystem(
      SessionModel.SessionId sessionId, SessionModel.SessionMountId mountId) {
    if (!allowFile) throw new IllegalArgumentException("FILE_ACCESS_DENIED");
    Session session = require(sessionId);
    FilesystemMount mount = mountRegistry.entries.get(mountId.value());
    if (mount == null) throw new IllegalArgumentException("NO_FILESYSTEM " + mountId);
    Long current = mountRegistry.sessionAttachments.get(sessionId.value());
    if (current != null && current == mountId.value()) return;

    session.attachFilesystem(new Session.AttachedFilesystem(mountId, mount.provider));
    if (current != null) decrementMount(current);
    mount.attachments++;
    mountRegistry.sessionAttachments.put(sessionId.value(), mountId.value());
  }

  synchronized void detachFilesystem(SessionModel.SessionId sessionId) {
    Session session = require(sessionId);
    SessionModel.SessionMountId released = session.detachFilesystem();
    if (released != null) releaseMount(sessionId, released);
  }

  SessionModel.SessionMountId filesystem(SessionModel.SessionId sessionId) {
    return require(sessionId).filesystemMount();
  }

  synchronized FilesystemInfo filesystemInfo(SessionModel.SessionMountId mountId) {
    FilesystemMount mount = mountRegistry.entries.get(mountId.value());
    if (mount == null) throw new IllegalArgumentException("NO_FILESYSTEM " + mountId);
    return new FilesystemInfo("native", mount.root, mount.attachments);
  }

  synchronized void closeFilesystem(SessionModel.SessionMountId mountId) {
    FilesystemMount mount = mountRegistry.entries.get(mountId.value());
    if (mount == null) throw new IllegalArgumentException("NO_FILESYSTEM " + mountId);
    if (mount.attachments != 0) {
      throw new IllegalArgumentException("FILESYSTEM_ATTACHED " + mountId);
    }
    mountRegistry.entries.remove(mountId.value());
  }

  synchronized void mountFilesystem(SessionModel.SessionId sessionId, Path root) {
    SessionModel.SessionMountId previous = filesystem(sessionId);
    SessionModel.SessionMountId created = createFilesystem(root);
    try {
      attachFilesystem(sessionId, created);
    } catch (RuntimeException error) {
      closeFilesystem(created);
      throw error;
    }
    if (previous != null) closeFilesystem(previous);
  }

  private synchronized void releaseMount(
      SessionModel.SessionId sessionId, SessionModel.SessionMountId mountId) {
    Long registered = mountRegistry.sessionAttachments.get(sessionId.value());
    if (registered == null || registered != mountId.value()) return;
    mountRegistry.sessionAttachments.remove(sessionId.value());
    decrementMount(mountId.value());
  }

  private void decrementMount(long mountId) {
    FilesystemMount mount = mountRegistry.entries.get(mountId);
    if (mount != null && mount.attachments > 0) mount.attachments--;
  }

  synchronized void closeSession(SessionModel.SessionId id) {
    if (ROOT_ID.equals(id)) throw new IllegalArgumentException("ROOT_CANNOT_CLOSE");
    Session removed = sessionRegistry.entries.remove(id.value());
    if (removed == null) throw new IllegalArgumentException("NO_SESSION " + id);
    removed.close();
  }

  Set<SessionModel.SessionId> sessionIds() {
    java.util.HashSet<SessionModel.SessionId> ids = new java.util.HashSet<>();
    for (Session session : sessionRegistry.entries.values()) ids.add(session.id());
    return Collections.unmodifiableSet(ids);
  }

  int size() {
    return sessionRegistry.entries.size();
  }

  void registerDevelopmentResource(String name, String source) {
    developmentResources.entries.put(name, source);
  }

  boolean removeDevelopmentResource(String name) {
    return developmentResources.entries.remove(name) != null;
  }

  Set<String> developmentResourceNames() {
    return Collections.unmodifiableSet(new java.util.TreeSet<>(developmentResources.entries.keySet()));
  }

  synchronized void registerBundle(String digest, byte[] bytes) {
    byte[] frozen = Arrays.copyOf(bytes, bytes.length);
    byte[] current = bundleCatalog.entries.get(digest);
    if (current != null && !Arrays.equals(current, frozen)) {
      throw new IllegalArgumentException("BUNDLE_DIGEST_CONFLICT " + digest);
    }
    if (current == null) bundleCatalog.entries.put(digest, frozen);
  }

  byte[] bundle(String digest) {
    byte[] bytes = bundleCatalog.entries.get(digest);
    return bytes == null ? null : Arrays.copyOf(bytes, bytes.length);
  }

  private static final class Sandbox {
    final SandboxModel.SandboxId id;
    final String provider;
    final boolean secure;
    final SessionModel.SessionMountId mount;
    final SandboxProvider.SandboxInstance instance;
    private final AtomicLong nextEvaluationId = new AtomicLong(1);

    Sandbox(
        SandboxModel.SandboxId id,
        String provider,
        boolean secure,
        SessionModel.SessionMountId mount,
        SandboxProvider.SandboxInstance instance) {
      this.id = id;
      this.provider = provider;
      this.secure = secure;
      this.mount = mount;
      this.instance = instance;
    }

    SandboxModel.EvaluationId allocateEvaluation() {
      long value = nextEvaluationId.getAndIncrement();
      if (value <= 0) throw new IllegalStateException("SANDBOX_EVALUATION_IDS_EXHAUSTED");
      return new SandboxModel.EvaluationId(value);
    }
  }

  void registerSandboxProvider(SandboxProvider provider) {
    sandboxProviderRegistry.entries.put(provider.name(), provider);
  }

  synchronized SandboxModel.SandboxId openSandbox(SandboxModel.SandboxSpec spec) {
    SandboxProvider provider = sandboxProviderRegistry.entries.get(spec.provider());
    if (provider == null) {
      throw new SandboxModel.SandboxException(
          SandboxModel.ErrorCode.PROVIDER_NOT_FOUND, spec.provider());
    }
    long value = sandboxRegistry.nextId.getAndIncrement();
    if (value <= 0) throw new IllegalStateException("SANDBOX_IDS_EXHAUSTED");
    SandboxModel.SandboxId id = new SandboxModel.SandboxId(value);
    java.util.LinkedHashMap<String, byte[]> bundles = new java.util.LinkedHashMap<>();
    for (SandboxModel.BundleReference reference : spec.bundles()) {
      byte[] bytes = bundle(reference.digest());
      if (bytes == null) {
        throw new SandboxModel.SandboxException(
            SandboxModel.ErrorCode.BUNDLE_NOT_FOUND, reference.digest());
      }
      if (!reference.digest().equals(sha256Digest(bytes))) {
        throw new SandboxModel.SandboxException(
            SandboxModel.ErrorCode.BUNDLE_DIGEST_MISMATCH, reference.digest());
      }
      bundles.put(reference.digest(), bytes);
    }
    FilesystemMount mount = null;
    if (spec.mount() != null) {
      mount = mountRegistry.entries.get(spec.mount().value());
      if (mount == null) {
        throw new SandboxModel.SandboxException(
            SandboxModel.ErrorCode.MOUNT_NOT_FOUND, spec.mount().toString());
      }
      mount.attachments++;
      mountRegistry.sandboxAttachments.put(id.value(), spec.mount().value());
    }
    try {
      SandboxProvider.ResolvedSpec resolved =
          new SandboxProvider.ResolvedSpec(
              spec,
              java.util.Collections.unmodifiableMap(bundles),
              mount == null ? null : mount.provider);
      sandboxRegistry.entries.put(
          value,
          new Sandbox(id, provider.name(), provider.secure(), spec.mount(), provider.open(resolved)));
    } catch (RuntimeException error) {
      releaseSandboxMount(id, spec.mount());
      throw error;
    }
    return id;
  }

  private static String sha256Digest(byte[] bytes) {
    try {
      byte[] digest = java.security.MessageDigest.getInstance("SHA-256").digest(bytes);
      return "sha256:" + java.util.HexFormat.of().formatHex(digest);
    } catch (java.security.NoSuchAlgorithmException error) {
      throw new IllegalStateException("SHA-256 is required", error);
    }
  }

  private void releaseSandboxMount(
      SandboxModel.SandboxId sandboxId, SessionModel.SessionMountId mountId) {
    if (mountId == null) return;
    Long registered = mountRegistry.sandboxAttachments.remove(sandboxId.value());
    if (registered != null && registered == mountId.value()) decrementMount(registered);
  }

  private Sandbox requireSandbox(SandboxModel.SandboxId id) {
    Sandbox sandbox = sandboxRegistry.entries.get(id.value());
    if (sandbox == null) {
      throw new SandboxModel.SandboxException(SandboxModel.ErrorCode.NOT_FOUND, id.toString());
    }
    return sandbox;
  }

  SandboxProvider.Pending<Object> sandboxEval(SandboxModel.SandboxId id, String source) {
    Sandbox sandbox = requireSandbox(id);
    return sandbox.instance.eval(sandbox.allocateEvaluation(), source);
  }

  SandboxProvider.Pending<Object> sandboxCall(
      SandboxModel.SandboxId id, String callable, java.util.List<Object> arguments) {
    Sandbox sandbox = requireSandbox(id);
    return sandbox.instance.call(sandbox.allocateEvaluation(), callable, arguments);
  }

  boolean cancelSandbox(SandboxModel.SandboxId id) {
    SandboxProvider.SandboxInstance instance = requireSandbox(id).instance;
    SandboxModel.EvaluationId evaluation = instance.activeEvaluation();
    return evaluation != null && instance.cancel(evaluation);
  }

  SandboxModel.SandboxStatus sandboxStatus(SandboxModel.SandboxId id) {
    Sandbox sandbox = requireSandbox(id);
    return new SandboxModel.SandboxStatus(
        sandbox.id,
        sandbox.provider,
        sandbox.instance.state(),
        sandbox.secure,
        sandbox.instance.activeEvaluation() != null,
        sandbox.instance.error());
  }

  synchronized void closeSandbox(SandboxModel.SandboxId id) {
    Sandbox sandbox = sandboxRegistry.entries.remove(id.value());
    if (sandbox == null) {
      throw new SandboxModel.SandboxException(SandboxModel.ErrorCode.NOT_FOUND, id.toString());
    }
    try {
      sandbox.instance.close();
    } finally {
      releaseSandboxMount(id, sandbox.mount);
    }
  }

  @Override
  public synchronized void close() {
    EMBEDDINGS.remove(embeddingToken, this);
    for (Sandbox sandbox : sandboxRegistry.entries.values()) sandbox.instance.close();
    sandboxRegistry.entries.clear();
    for (Session session : sessionRegistry.entries.values()) session.close();
    sessionRegistry.entries.clear();
    mountRegistry.sessionAttachments.clear();
    mountRegistry.sandboxAttachments.clear();
    mountRegistry.entries.clear();
  }

  static final class Session
      implements AutoCloseable, IContext, IComponent, IApplicable, IInvokeIn {
    private final SessionModel.SessionSpec spec;
    private final SessionAuthorityPolicy authority;
    private final HaraProject project;
    private final Consumer<SessionModel.SessionMountId> mountRelease;
    private final boolean sandboxRestricted;
    private final String kernelToken;
    private Context context;
    private volatile AttachedFilesystem filesystem;
    private final AtomicInteger activeEvaluations = new AtomicInteger();
    private final AtomicReference<SessionModel.SessionState> state =
        new AtomicReference<>(SessionModel.SessionState.NEW);

    private record AttachedFilesystem(
        SessionModel.SessionMountId id, HaraMountedFileSystem provider) {}

    private Session(
        SessionModel.SessionSpec spec,
        HaraProject project,
        Consumer<SessionModel.SessionMountId> mountRelease,
        boolean sandboxRestricted,
        String kernelToken) {
      this.spec = spec;
      this.authority = spec.authority();
      this.project = project;
      this.mountRelease = mountRelease;
      this.sandboxRestricted = sandboxRestricted;
      this.kernelToken = kernelToken;
      context = createContext(null);
      activate();
    }

    static Session privateSandbox(String entryNamespace) {
      Session session =
          new Session(
              SessionModel.SessionSpec.zeroAuthority(SessionModel.SessionId.parse("SANDBOX")),
              null,
              ignored -> {},
              true,
              null);
      if (!"user".equals(entryNamespace)) session.eval("(ns " + entryNamespace + ")");
      return session;
    }

    void attachSandboxFilesystem(
        SessionModel.SessionMountId mountId, HaraMountedFileSystem provider) {
      attachFilesystem(new AttachedFilesystem(mountId, provider));
    }

    private Context createContext(AttachedFilesystem filesystem) {
      IOAccess.Builder io = IOAccess.newBuilder().allowHostSocketAccess(authority.hostNetwork);
      if (filesystem == null) {
        io.allowHostFileAccess(authority.hostFilesystem);
      } else {
        io.allowHostFileAccess(false).fileSystem(filesystem.provider());
      }
      Context.Builder builder =
          Context.newBuilder(HaraLanguage.ID)
              .option("hara.SandboxRestricted", Boolean.toString(sandboxRestricted))
              .allowCreateProcess(authority.hostProcess)
              .allowIO(io.build());
      if (kernelToken != null) builder.option("hara.KernelToken", kernelToken);
      if (authority.project && project != null && filesystem == null) {
        builder.currentWorkingDirectory(project.root());
      }
      if (authority.reflection && project != null) {
        builder.allowHostAccess(HostAccess.ALL).allowHostClassLookup(name -> true);
      }
      return builder.build();
    }

    private void requireActive() {
      SessionModel.SessionState current = state.get();
      if (current == SessionModel.SessionState.CLOSED)
        throw new IllegalStateException("SESSION_CLOSED " + id());
      if (current != SessionModel.SessionState.ACTIVE)
        throw new IllegalStateException("SESSION_NOT_ACTIVE " + id() + " " + current);
    }

    void attachFilesystem(AttachedFilesystem attached) {
      requireActive();
      if (activeEvaluations.get() != 0) throw new IllegalArgumentException("SESSION_BUSY " + id());
      Context replacement = createContext(attached);
      synchronized (this) {
        if (state.get() != SessionModel.SessionState.ACTIVE) {
          replacement.close(true);
          requireActive();
        }
        if (activeEvaluations.get() != 0) {
          replacement.close(true);
          throw new IllegalArgumentException("SESSION_BUSY " + id());
        }
        Context previous = context;
        context = replacement;
        filesystem = attached;
        previous.close(true);
      }
    }

    SessionModel.SessionMountId detachFilesystem() {
      requireActive();
      if (activeEvaluations.get() != 0) throw new IllegalArgumentException("SESSION_BUSY " + id());
      Context replacement = createContext(null);
      synchronized (this) {
        if (state.get() != SessionModel.SessionState.ACTIVE) {
          replacement.close(true);
          requireActive();
        }
        if (activeEvaluations.get() != 0) {
          replacement.close(true);
          throw new IllegalArgumentException("SESSION_BUSY " + id());
        }
        Context previous = context;
        SessionModel.SessionMountId released = filesystemMount();
        context = replacement;
        filesystem = null;
        previous.close(true);
        return released;
      }
    }

    SessionModel.SessionId id() {
      return spec.id();
    }

    String name() {
      return id().value();
    }

    SessionModel.SessionState state() {
      return state.get();
    }

    SessionModel.SessionMountId filesystemMount() {
      AttachedFilesystem attached = filesystem;
      return attached == null ? null : attached.id();
    }

    SessionAuthorityPolicy authority() {
      return authority;
    }

    Value eval(String source) {
      return eval(source, null, 1, 1);
    }

    Object evalTransfer(String source) {
      return transferValue(eval(source));
    }

    Object callTransfer(String callable, List<Object> arguments) {
      activeEvaluations.incrementAndGet();
      try {
        synchronized (this) {
          requireActive();
          Value function = context.eval(HaraLanguage.ID, callable);
          if (!function.canExecute()) {
            throw new IllegalArgumentException("SESSION_VAR_NOT_CALLABLE " + callable);
          }
          Value result = function.execute(arguments.toArray());
          return transferValue(result);
        }
      } catch (PolyglotException error) {
        throw new IllegalArgumentException(error.getMessage(), error);
      } finally {
        activeEvaluations.decrementAndGet();
      }
    }

    void cancelEvaluation() {
      Context active = context;
      if (active != null) active.close(true);
    }

    private static Object transferValue(Value value) {
      if (value.isNull()) return null;
      if (value.isBoolean()) return value.asBoolean();
      if (value.isString()) return value.asString();
      if (value.fitsInLong()) return value.asLong();
      if (value.fitsInDouble()) return value.asDouble();
      String display = value.toString();
      if (display.contains("#'")
          || display.contains("#atom")
          || display.contains("#<")
          || display.contains("#object")
          || display.contains("#array")
          || display.contains("#bytes")
          || display.contains("@")) {
        throw new IllegalArgumentException("SESSION_TRANSFER_REJECTED " + display);
      }
      if (value.hasIterator() && display.startsWith("#{")) {
        java.util.LinkedHashSet<Object> transferred = new java.util.LinkedHashSet<>();
        Value iterator = value.getIterator();
        while (iterator.hasIteratorNextElement()) {
          transferred.add(transferValue(iterator.getIteratorNextElement()));
        }
        return HaraPersistentValues.normalize(transferred);
      }
      if (value.hasArrayElements()) {
        java.util.ArrayList<Object> transferred = new java.util.ArrayList<>();
        for (long index = 0; index < value.getArraySize(); index++) {
          transferred.add(transferValue(value.getArrayElement(index)));
        }
        return HaraPersistentValues.normalize(transferred);
      }
      if (value.hasHashEntries()) {
        java.util.LinkedHashMap<Object, Object> transferred = new java.util.LinkedHashMap<>();
        Value entries = value.getHashEntriesIterator();
        while (entries.hasIteratorNextElement()) {
          Value entry = entries.getIteratorNextElement();
          transferred.put(
              transferValue(entry.getArrayElement(0)), transferValue(entry.getArrayElement(1)));
        }
        return HaraPersistentValues.normalize(transferred);
      }
      if (value.hasIterator()) {
        throw new IllegalArgumentException("SESSION_TRANSFER_REJECTED " + display);
      }
      try {
        Object[] forms = HaraLanguage.readAll(display, "<session-transfer>");
        if (forms.length != 1) {
          throw new IllegalArgumentException("SESSION_TRANSFER_REJECTED " + display);
        }
        return forms[0];
      } catch (RuntimeException error) {
        if (error instanceof IllegalArgumentException
            && error.getMessage() != null
            && error.getMessage().startsWith("SESSION_TRANSFER_REJECTED")) {
          throw error;
        }
        throw new IllegalArgumentException(
            "SESSION_TRANSFER_REJECTED "
                + display
                + " ("
                + error.getClass().getSimpleName()
                + ": "
                + error.getMessage()
                + ")",
            error);
      }
    }

    Value eval(String source, String file, int line, int column) {
      activeEvaluations.incrementAndGet();
      try {
        synchronized (this) {
          requireActive();
          if (file == null || file.isBlank()) return context.eval(HaraLanguage.ID, source);
          int safeLine = Math.max(1, line);
          int safeColumn = Math.max(1, column);
          StringBuilder contextual = new StringBuilder(source.length() + safeLine + safeColumn);
          contextual.append("\n".repeat(safeLine - 1));
          contextual.append(" ".repeat(safeColumn - 1));
          contextual.append(source);
          Source contextualSource =
              Source.newBuilder(HaraLanguage.ID, contextual.toString(), file).build();
          return context.eval(contextualSource);
        }
      } catch (IOException error) {
        throw new IllegalArgumentException(
            "Unable to construct Hara source: " + error.getMessage(), error);
      } catch (PolyglotException error) {
        throw new IllegalArgumentException(error.getMessage(), error);
      } finally {
        activeEvaluations.decrementAndGet();
      }
    }

    synchronized String currentNamespace() {
      Value value = eval("(current-namespace)");
      return value.isString() ? value.asString() : value.toString();
    }

    synchronized List<String> currentSymbols() {
      Value values = eval("(current-symbols)");
      List<String> result = new ArrayList<>();
      for (long index = 0; index < values.getArraySize(); index++) {
        result.add(values.getArrayElement(index).asString());
      }
      return result;
    }

    List<Object> info() {
      String filesystem =
          this.filesystem == null
              ? (authority.hostFilesystem ? "HOST" : "DENIED")
              : this.filesystem.id().toString();
      return List.of(
          "NAME", name(),
          "STATE", state().toString().toUpperCase(java.util.Locale.ROOT),
          "FILESYSTEM", filesystem,
          "AUTHORITY", authority.profile());
    }

    @Override
    public Object call(Object... args) {
      requireActive();
      if (args == null || args.length != 1 || !(args[0] instanceof String)) {
        throw new IllegalArgumentException("SESSION_CALL_EXPECTS_SOURCE " + id());
      }
      return evalTransfer((String) args[0]);
    }

    @Override
    public IMetadata getProps() {
      return metadata();
    }

    @Override
    public IMetadata getStatus() {
      return metadata();
    }

    private SessionModel.SessionStatus metadata() {
      boolean running = state.get() == SessionModel.SessionState.ACTIVE;
      return new SessionModel.SessionStatus(
          id(), running ? currentNamespace() : null, state(), filesystemMount(), authority);
    }

    @Override
    public boolean isStarted() {
      return state.get() == SessionModel.SessionState.ACTIVE;
    }

    @Override
    public boolean isStopped() {
      return state.get() == SessionModel.SessionState.CLOSED;
    }

    @Override
    public IComponent start() {
      requireActive();
      return this;
    }

    @Override
    public IComponent stop() {
      close();
      return this;
    }

    @Override
    public Object applyDefault() {
      return this;
    }

    @Override
    public Object applyIn(Object runtime, Object[] args) {
      requireActive();
      if (!(runtime instanceof IContext)) {
        throw new IllegalArgumentException("SESSION_APPLY_EXPECTS_CONTEXT " + id());
      }
      return ((IContext) runtime).call(args == null ? new Object[0] : args);
    }

    @Override
    public Object transformIn(Object runtime, Object[] args) {
      return args;
    }

    @Override
    public Object transformOut(Object runtime, Object[] args, Object value) {
      return value;
    }

    @Override
    public Object invokeIn(IContext context, Object... args) {
      return applyIn(context, args);
    }

    @Override
    public void close() {
      if (!state.compareAndSet(SessionModel.SessionState.ACTIVE, SessionModel.SessionState.CLOSED))
        return;
      Context ownedContext;
      AttachedFilesystem ownedFilesystem;
      synchronized (this) {
        ownedContext = context;
        ownedFilesystem = filesystem;
        context = null;
        filesystem = null;
      }
      try {
        if (ownedContext != null) ownedContext.close(true);
      } finally {
        if (ownedFilesystem != null) mountRelease.accept(ownedFilesystem.id());
      }
    }

    private void activate() {
      if (!state.compareAndSet(SessionModel.SessionState.NEW, SessionModel.SessionState.ACTIVE)) {
        throw new IllegalStateException("SESSION_ALREADY_STARTED " + id());
      }
    }
  }
}
