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
  private final ConcurrentHashMap<String, Session> sessions = new ConcurrentHashMap<>();
  private final ConcurrentHashMap<Long, FilesystemMount> mounts = new ConcurrentHashMap<>();
  private final ConcurrentHashMap<String, Long> sessionMounts = new ConcurrentHashMap<>();
  private final AtomicLong nextMountId = new AtomicLong(1);
  private static final SessionModel.SessionId ROOT_ID = SessionModel.SessionId.parse("ROOT");

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
    SessionAuthorityPolicy rootAuthority =
        SessionAuthorityPolicy.root(allowFile, allowNetwork, allowProcess, project);
    sessions.put(
        ROOT_ID.value(),
        new Session(
            new SessionModel.SessionSpec(ROOT_ID, rootAuthority),
            project,
            mount -> releaseMount(ROOT_ID, mount)));
  }

  Session root() {
    return require(ROOT_ID);
  }

  Session require(SessionModel.SessionId id) {
    Session session = sessions.get(id.value());
    if (session == null) throw new IllegalArgumentException("NO_SESSION " + id);
    return session;
  }

  synchronized Session create(SessionModel.SessionId id) {
    if (sessions.containsKey(id.value()))
      throw new IllegalArgumentException("SESSION_EXISTS " + id);
    Session session =
        new Session(
            SessionModel.SessionSpec.zeroAuthority(id), null, mount -> releaseMount(id, mount));
    sessions.put(id.value(), session);
    return session;
  }

  synchronized SessionModel.SessionMountId createFilesystem(Path root) {
    if (!allowFile) throw new IllegalArgumentException("FILE_ACCESS_DENIED");
    Path normalized = root.toAbsolutePath().normalize();
    if (!Files.isDirectory(normalized)) {
      throw new IllegalArgumentException("FILESYSTEM_NOT_FOUND " + normalized);
    }
    long value = nextMountId.getAndIncrement();
    if (value <= 0) throw new IllegalStateException("FILESYSTEM_IDS_EXHAUSTED");
    SessionModel.SessionMountId id = SessionModel.SessionMountId.of(value);
    mounts.put(value, new FilesystemMount(new HaraMountedFileSystem(normalized), normalized));
    return id;
  }

  synchronized void attachFilesystem(
      SessionModel.SessionId sessionId, SessionModel.SessionMountId mountId) {
    if (!allowFile) throw new IllegalArgumentException("FILE_ACCESS_DENIED");
    Session session = require(sessionId);
    FilesystemMount mount = mounts.get(mountId.value());
    if (mount == null) throw new IllegalArgumentException("NO_FILESYSTEM " + mountId);
    Long current = sessionMounts.get(sessionId.value());
    if (current != null && current == mountId.value()) return;

    session.attachFilesystem(new Session.AttachedFilesystem(mountId, mount.provider));
    if (current != null) decrementMount(current);
    mount.attachments++;
    sessionMounts.put(sessionId.value(), mountId.value());
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
    FilesystemMount mount = mounts.get(mountId.value());
    if (mount == null) throw new IllegalArgumentException("NO_FILESYSTEM " + mountId);
    return new FilesystemInfo("native", mount.root, mount.attachments);
  }

  synchronized void closeFilesystem(SessionModel.SessionMountId mountId) {
    FilesystemMount mount = mounts.get(mountId.value());
    if (mount == null) throw new IllegalArgumentException("NO_FILESYSTEM " + mountId);
    if (mount.attachments != 0) {
      throw new IllegalArgumentException("FILESYSTEM_ATTACHED " + mountId);
    }
    mounts.remove(mountId.value());
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
    Long registered = sessionMounts.get(sessionId.value());
    if (registered == null || registered != mountId.value()) return;
    sessionMounts.remove(sessionId.value());
    decrementMount(mountId.value());
  }

  private void decrementMount(long mountId) {
    FilesystemMount mount = mounts.get(mountId);
    if (mount != null && mount.attachments > 0) mount.attachments--;
  }

  synchronized void closeSession(SessionModel.SessionId id) {
    if (ROOT_ID.equals(id)) throw new IllegalArgumentException("ROOT_CANNOT_CLOSE");
    Session removed = sessions.remove(id.value());
    if (removed == null) throw new IllegalArgumentException("NO_SESSION " + id);
    removed.close();
  }

  Set<SessionModel.SessionId> sessionIds() {
    java.util.HashSet<SessionModel.SessionId> ids = new java.util.HashSet<>();
    for (Session session : sessions.values()) ids.add(session.id());
    return Collections.unmodifiableSet(ids);
  }

  int size() {
    return sessions.size();
  }

  @Override
  public synchronized void close() {
    for (Session session : sessions.values()) session.close();
    sessions.clear();
    sessionMounts.clear();
    mounts.clear();
  }

  static final class Session
      implements AutoCloseable, IContext, IComponent, IApplicable, IInvokeIn {
    private final SessionModel.SessionSpec spec;
    private final SessionAuthorityPolicy authority;
    private final HaraProject project;
    private final Consumer<SessionModel.SessionMountId> mountRelease;
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
        Consumer<SessionModel.SessionMountId> mountRelease) {
      this.spec = spec;
      this.authority = spec.authority();
      this.project = project;
      this.mountRelease = mountRelease;
      context = createContext(null);
      activate();
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
              .allowCreateProcess(authority.hostProcess)
              .allowIO(io.build());
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
