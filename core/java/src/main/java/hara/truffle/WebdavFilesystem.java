package hara.truffle;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.HashMap;
import java.util.HashSet;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Set;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CompletionException;
import java.util.concurrent.CompletionStage;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.Executor;
import java.util.concurrent.ScheduledExecutorService;
import java.util.concurrent.ScheduledFuture;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.function.Supplier;

/**
 * Provider-neutral WebDAV mount over a trusted authenticated transport capability.
 *
 * <p>The public nested transport/factory types are trusted Java provider SPI for the package
 * migration. They are not exposed to Hara values. The implementation remains in the base runtime
 * during the compatibility phase and is relocated into the provider artifact only after the
 * package-loading pilot is proven.
 */
public final class WebdavFilesystem implements IFilesystem {
  /** Trusted host transport. Authentication happens before Hara receives this capability. */
  public interface Client extends AutoCloseable {
    boolean authenticated();

    default boolean transportVerified() {
      return true;
    }

    Set<Capability> capabilities();

    RemoteEntry lstat(String path) throws Exception;

    byte[] read(String path, long maxBytes) throws Exception;

    void write(String path, byte[] bytes, WriteMode mode, MutationContext mutation)
        throws Exception;

    List<RemoteEntry> entries(String path) throws Exception;

    void mkdir(String path, MutationContext mutation) throws Exception;

    void delete(String path, boolean directory, MutationContext mutation) throws Exception;

    void move(
        String source,
        String target,
        boolean replace,
        boolean atomic,
        MutationContext mutation)
        throws Exception;

    @Override
    void close() throws Exception;
  }

  public record RemoteEntry(
      String name,
      EntryType type,
      Long size,
      Long modifiedAt,
      String id,
      String revision,
      Capabilities capabilities,
      Map<String, Object> extensions) {
    public RemoteEntry {
      if (name == null
          || name.isBlank()
          || name.contains("/")
          || ".".equals(name)
          || "..".equals(name)) {
        throw new IllegalArgumentException("invalid WebDAV entry name");
      }
      type = Objects.requireNonNull(type, "WebDAV entry type");
      if (size != null && size < 0) throw new IllegalArgumentException("negative WebDAV entry size");
      extensions = extensions == null || extensions.isEmpty() ? Map.of() : Map.copyOf(extensions);
    }
  }

  /** Typed transport failure. Portable behavior never depends on server message strings. */
  public static final class ClientFailure extends Exception {
    private static final long serialVersionUID = 1L;
    private final String code;
    private final String providerCode;
    private final boolean retryable;

    public ClientFailure(String code, String providerCode, boolean retryable) {
      super("WebDAV transport operation failed");
      this.code = requireCode(code);
      this.providerCode = providerCode;
      this.retryable = retryable;
    }

    public String code() {
      return code;
    }

    public String providerCode() {
      return providerCode;
    }

    public boolean retryable() {
      return retryable;
    }

    private static String requireCode(String value) {
      if (value == null || !value.matches("[a-z][a-z0-9-]*")) {
        throw new IllegalArgumentException("invalid WebDAV failure code");
      }
      return value;
    }
  }

  public static final class Factory implements IFilesystemFactory {
    private static final Set<String> ALLOWED =
        Set.of(
            "credential-ref",
            "root",
            "root-url",
            "read-only?",
            "display",
            "operation-timeout-ms",
            "max-transfer-bytes");

    @Override
    public String kind() {
      return "webdav";
    }

    @Override
    public void validate(Map<String, ?> configuration) {
      IFilesystemFactory.super.validate(configuration);
      for (String key : configuration.keySet()) {
        if (!ALLOWED.contains(key)) {
          throw new IllegalArgumentException("unknown WebDAV filesystem option " + key);
        }
      }
      requireText(configuration, "credential-ref");
      requireUrl(resolveRootValue(configuration), "root-url");
      Object readOnly = configuration.get("read-only?");
      if (readOnly != null && !(readOnly instanceof Boolean)) {
        throw new IllegalArgumentException("WebDAV filesystem read-only? must be a boolean");
      }
      Object display = configuration.get("display");
      if (display != null && (!(display instanceof String text) || text.isBlank())) {
        throw new IllegalArgumentException("WebDAV filesystem display must be a nonblank string");
      }
      positiveLong(configuration, "operation-timeout-ms", 30_000L);
      positiveLong(configuration, "max-transfer-bytes", 16L * 1024L * 1024L);
    }

    @Override
    public CompletionStage<IFilesystem> open(OpenContext context, Map<String, ?> configuration) {
      Objects.requireNonNull(context, "filesystem open context");
      validate(configuration);
      String credentialReference = (String) configuration.get("credential-ref");
      Object resolved = context.credentials().resolve(credentialReference);
      if (!(resolved instanceof Client client)) {
        return CompletableFuture.failedFuture(
            new IllegalArgumentException(
                "WebDAV credential reference did not resolve to a trusted WebDAV client"));
      }
      if (!client.authenticated() || !client.transportVerified()) {
        return CompletableFuture.failedFuture(
            failure(
                "authentication-failed",
                "WebDAV transport is not authenticated and verified",
                "open",
                null,
                null,
                "transport-unverified",
                false,
                null));
      }
      String root = requireUrl(resolveRootValue(configuration), "root-url");
      boolean readOnly = Boolean.TRUE.equals(configuration.get("read-only?"));
      String display =
          configuration.get("display") instanceof String value ? value : "WebDAV filesystem";
      long operationTimeoutMillis = positiveLong(configuration, "operation-timeout-ms", 30_000L);
      long maxTransferBytes = positiveLong(configuration, "max-transfer-bytes", 16L * 1024L * 1024L);
      WebdavFilesystem filesystem =
          new WebdavFilesystem(
              client,
              root,
              display,
              readOnly,
              operationTimeoutMillis,
              maxTransferBytes,
              context.ioExecutor(),
              context.scheduler());
      return filesystem.submit(
          IFilesystem.CallContext.create(),
          "open",
          "/",
          null,
          () -> {
            RemoteEntry entry = filesystem.client.lstat(root);
            if (entry.type() == EntryType.SYMLINK) {
              throw failure(
                  "outside-root",
                  "WebDAV root cannot be a symbolic link",
                  "open",
                  "/",
                  null,
                  "root-symlink",
                  false,
                  null);
            }
            if (entry.type() != EntryType.DIRECTORY) {
              throw failure(
                  "not-directory",
                  "WebDAV root is not a directory",
                  "open",
                  "/",
                  null,
                  "root-not-directory",
                  false,
                  null);
            }
            return (IFilesystem) filesystem;
          });
    }

    private static String requireText(Map<String, ?> values, String key) {
      Object value = values.get(key);
      if (!(value instanceof String text) || text.isBlank()) {
        throw new IllegalArgumentException("WebDAV filesystem " + key + " is required");
      }
      return text;
    }

    private static String resolveRootValue(Map<String, ?> values) {
      String root = values.get("root-url") instanceof String value && !value.isBlank() ? value : null;
      if (root == null && values.get("root") instanceof String value && !value.isBlank()) root = value;
      if (root == null) throw new IllegalArgumentException("WebDAV filesystem root-url is required");
      return root;
    }

    private static String requireUrl(String value, String key) {
      String text = requireText(Map.of(key, value), key);
      try {
        java.net.URI uri = java.net.URI.create(text);
        String scheme = uri.getScheme();
        if (scheme == null
            || !("http".equalsIgnoreCase(scheme) || "https".equalsIgnoreCase(scheme))) {
          throw new IllegalArgumentException("WebDAV filesystem " + key + " must be an http(s) URL");
        }
        return text;
      } catch (IllegalArgumentException error) {
        if (error.getMessage() != null && error.getMessage().contains("WebDAV filesystem ")) throw error;
        throw new IllegalArgumentException("WebDAV filesystem " + key + " must be an http(s) URL");
      }
    }

    private static long positiveLong(Map<String, ?> values, String key, long fallback) {
      Object value = values.get(key);
      if (value == null) return fallback;
      if (!(value instanceof Number number) || number.longValue() <= 0) {
        throw new IllegalArgumentException("WebDAV filesystem " + key + " must be positive");
      }
      return number.longValue();
    }
  }

  private record Pending(CompletableFuture<?> future, String operation, String path, String target) {}

  private final Client client;
  private final String root;
  private final String display;
  private final boolean readOnly;
  private final long operationTimeoutMillis;
  private final long maxTransferBytes;
  private final Executor ioExecutor;
  private final ScheduledExecutorService scheduler;
  private final Set<Capability> transportCapabilities;
  private final Capabilities capabilities;
  private final AtomicBoolean closed = new AtomicBoolean();
  private final Set<Pending> pending = ConcurrentHashMap.newKeySet();

  WebdavFilesystem(
      Client client,
      String root,
      String display,
      boolean readOnly,
      long operationTimeoutMillis,
      long maxTransferBytes,
      Executor ioExecutor,
      ScheduledExecutorService scheduler) {
    this.client = Objects.requireNonNull(client, "WebDAV client");
    this.root = rootUrl(root);
    this.display = Objects.requireNonNull(display, "WebDAV display");
    this.readOnly = readOnly;
    this.operationTimeoutMillis = operationTimeoutMillis;
    this.maxTransferBytes = maxTransferBytes;
    this.ioExecutor = Objects.requireNonNull(ioExecutor, "filesystem I/O executor");
    this.scheduler = Objects.requireNonNull(scheduler, "filesystem scheduler");
    this.transportCapabilities = Set.copyOf(client.capabilities());
    this.capabilities = new Capabilities(advertisedCapabilities());
  }

  @Override
  public Descriptor descriptor() {
    return new Descriptor(
        "webdav",
        display,
        readOnly,
        capabilities,
        null,
        Map.of("provider/root-scoped?", true, "provider/transport-verified?", true));
  }

  @Override
  public CompletionStage<Entry> stat(CallContext context, String path) {
    String logical = normalise(path);
    return submit(
        context,
        "stat",
        logical,
        null,
        () -> {
          require(Capability.READ, "stat", logical, null);
          guardAncestors(logical, false, "stat", null);
          return entry(logical, client.lstat(remote(logical)));
        });
  }

  @Override
  public CompletionStage<byte[]> read(CallContext context, String path) {
    String logical = normalise(path);
    return submit(
        context,
        "read",
        logical,
        null,
        () -> {
          require(Capability.READ, "read", logical, null);
          guardAncestors(logical, false, "read", null);
          RemoteEntry value = client.lstat(remote(logical));
          requireRegular(value, "read", logical, null);
          if (value.size() != null && value.size() > maxTransferBytes) {
            throw failure(
                "quota-exceeded",
                "WebDAV file exceeds configured transfer limit",
                "read",
                logical,
                null,
                "transfer-limit",
                false,
                null);
          }
          byte[] bytes = client.read(remote(logical), maxTransferBytes);
          if ((long) bytes.length > maxTransferBytes) {
            throw failure(
                "quota-exceeded",
                "WebDAV file exceeds configured transfer limit",
                "read",
                logical,
                null,
                "transfer-limit",
                false,
                null);
          }
          return bytes.clone();
        });
  }

  @Override
  public CompletionStage<Mutation> write(
      CallContext context,
      String path,
      byte[] bytes,
      WriteOptions options,
      MutationContext mutation) {
    String logical = normalise(path);
    byte[] copy = Objects.requireNonNull(bytes, "filesystem bytes").clone();
    Objects.requireNonNull(options, "write options");
    Objects.requireNonNull(mutation, "mutation context");
    return submit(
        context,
        "write",
        logical,
        null,
        () -> {
          requireWritable(Capability.WRITE, "write", logical, null);
          requireRevisionSupport(mutation, "write", logical, null);
          if (options.mode() == WriteMode.APPEND) require(Capability.APPEND, "write", logical, null);
          if ((long) copy.length > maxTransferBytes) {
            throw failure(
                "quota-exceeded",
                "WebDAV write exceeds configured transfer limit",
                "write",
                logical,
                null,
                "transfer-limit",
                false,
                null);
          }
          ensureParents(logical, options.parents(), "write");
          RemoteEntry existing = optionalLstat(remote(logical), "write", logical, null);
          if (existing != null) {
            if (existing.type() == EntryType.SYMLINK) throw unsupported("write", logical, null, "symlink-write");
            if (existing.type() == EntryType.DIRECTORY) {
              throw failure("is-directory", "path is a directory", "write", logical, null, null, false, null);
            }
          }
          client.write(remote(logical), copy, options.mode(), mutation);
          return mutation(logical, client.lstat(remote(logical)));
        });
  }

  @Override
  public CompletionStage<EntryPage> entriesPage(CallContext context, String path, PageRequest request) {
    String logical = normalise(path);
    Objects.requireNonNull(request, "filesystem page request");
    return submit(
        context,
        "entries",
        logical,
        null,
        () -> {
          require(Capability.ENTRIES, "entries", logical, null);
          guardAncestors(logical, false, "entries", null);
          RemoteEntry directory = client.lstat(remote(logical));
          if (directory.type() == EntryType.SYMLINK || directory.type() != EntryType.DIRECTORY) {
            throw failure("not-directory", "path is not a directory", "entries", logical, null, null, false, null);
          }
          ArrayList<Entry> entries = new ArrayList<>();
          for (RemoteEntry child : client.entries(remote(logical))) {
            entries.add(entry(HaraLogicalPath.join(logical, child.name()), child));
          }
          entries.sort(Comparator.comparing(Entry::path));
          int offset = pageOffset(request.token(), entries.size());
          int end = Math.min(entries.size(), offset + request.limit());
          String next = end < entries.size() ? Integer.toString(end) : null;
          return new EntryPage(entries.subList(offset, end), next);
        });
  }

  @Override
  public CompletionStage<Mutation> mkdir(CallContext context, String path, MkdirOptions options, MutationContext mutation) {
    String logical = normalise(path);
    Objects.requireNonNull(options, "mkdir options");
    Objects.requireNonNull(mutation, "mutation context");
    return submit(
        context,
        "mkdir",
        logical,
        null,
        () -> {
          requireWritable(Capability.MKDIR, "mkdir", logical, null);
          requireRevisionSupport(mutation, "mkdir", logical, null);
          if ("/".equals(logical)) {
            if (options.existsOk()) return Mutation.path("/");
            throw failure("already-exists", "mounted root already exists", "mkdir", logical, null, null, false, null);
          }
          RemoteEntry existing = optionalLstat(remote(logical), "mkdir", logical, null);
          if (existing != null) {
            if (existing.type() == EntryType.DIRECTORY && options.existsOk()) return mutation(logical, existing);
            throw failure("already-exists", "path already exists", "mkdir", logical, null, null, false, null);
          }
          ensureParents(logical, options.parents(), "mkdir");
          client.mkdir(remote(logical), mutation);
          return mutation(logical, client.lstat(remote(logical)));
        });
  }

  @Override
  public CompletionStage<Mutation> delete(CallContext context, String path, DeleteOptions options, MutationContext mutation) {
    String logical = normalise(path);
    Objects.requireNonNull(options, "delete options");
    Objects.requireNonNull(mutation, "mutation context");
    return submit(
        context,
        "delete",
        logical,
        null,
        () -> {
          requireWritable(Capability.DELETE, "delete", logical, null);
          requireRevisionSupport(mutation, "delete", logical, null);
          if ("/".equals(logical)) throw failure("denied", "cannot delete mounted root", "delete", logical, null, null, false, null);
          guardAncestors(logical, false, "delete", null);
          RemoteEntry existing = optionalLstat(remote(logical), "delete", logical, null);
          if (existing == null) {
            if (options.missingOk()) return Mutation.path(logical);
            throw failure("not-found", "path does not exist", "delete", logical, null, null, false, null);
          }
          if (existing.type() == EntryType.SYMLINK) throw unsupported("delete", logical, null, "symlink-delete");
          client.delete(remote(logical), existing.type() == EntryType.DIRECTORY, mutation);
          return Mutation.path(logical);
        });
  }

  @Override
  public CompletionStage<Mutation> copy(CallContext context, String source, String target, CopyOptions options, MutationContext mutation) {
    String from = normalise(source);
    String to = normalise(target);
    Objects.requireNonNull(options, "copy options");
    Objects.requireNonNull(mutation, "mutation context");
    return submit(
        context,
        "copy",
        from,
        to,
        () -> {
          requireWritable(Capability.COPY, "copy", from, to);
          throw unsupported("copy", from, to, "server-copy-unavailable");
        });
  }

  @Override
  public CompletionStage<Mutation> move(CallContext context, String source, String target, MoveOptions options, MutationContext mutation) {
    String from = normalise(source);
    String to = normalise(target);
    Objects.requireNonNull(options, "move options");
    Objects.requireNonNull(mutation, "mutation context");
    return submit(
        context,
        "move",
        from,
        to,
        () -> {
          requireWritable(Capability.MOVE, "move", from, to);
          requireRevisionSupport(mutation, "move", from, to);
          if (options.atomic()) throw unsupported("move", from, to, "atomic-move");
          guardAncestors(from, false, "move", to);
          guardAncestors(to, true, "move", from);
          RemoteEntry sourceEntry = client.lstat(remote(from));
          if (sourceEntry.type() == EntryType.SYMLINK) throw unsupported("move", from, to, "symlink-move");
          ensureParents(to, options.parents(), "move");
          RemoteEntry targetEntry = optionalLstat(remote(to), "move", from, to);
          if (targetEntry != null && !options.replace()) {
            throw failure("already-exists", "target already exists", "move", from, to, null, false, null);
          }
          client.move(remote(from), remote(to), options.replace(), false, mutation);
          return mutation(to, client.lstat(remote(to)));
        });
  }

  @Override
  public CompletionStage<Void> close(CallContext context) {
    if (!closed.compareAndSet(false, true)) return CompletableFuture.completedFuture(null);
    for (Pending operation : List.copyOf(pending)) operation.future().cancel(true);
    return CompletableFuture.runAsync(
        () -> {
          try {
            client.close();
          } catch (Exception error) {
            throw new CompletionException(mapFailure("close", "/", null, error));
          }
        },
        ioExecutor);
  }

  private Set<Capability> advertisedCapabilities() {
    HashSet<Capability> values = new HashSet<>();
    if (transportCapabilities.contains(Capability.READ)) values.add(Capability.READ);
    if (transportCapabilities.contains(Capability.ENTRIES)) values.add(Capability.ENTRIES);
    if (!readOnly) {
      for (Capability capability : List.of(Capability.WRITE, Capability.MKDIR, Capability.DELETE, Capability.MOVE)) {
        if (transportCapabilities.contains(capability)) values.add(capability);
      }
    }
    if (transportCapabilities.contains(Capability.REVISION_CHECK)) values.add(Capability.REVISION_CHECK);
    return values;
  }

  private void require(Capability capability, String operation, String path, String target) {
    if (!capabilities.contains(capability)) throw unsupported(operation, path, target, capability.keyword());
  }

  private void requireWritable(Capability capability, String operation, String path, String target) {
    if (readOnly) throw failure("permission-denied", "mounted WebDAV filesystem is read-only", operation, path, target, null, false, null);
    require(capability, operation, path, target);
  }

  private void requireRevisionSupport(MutationContext mutation, String operation, String path, String target) {
    if (mutation.required() && !capabilities.contains(Capability.REVISION_CHECK)) {
      throw FilesystemException.unsupportedRevision("webdav", operation, path, target);
    }
  }

  private void guardAncestors(String path, boolean allowMissing, String operation, String target) throws Exception {
    String parent = HaraLogicalPath.parent(path);
    ArrayList<String> ancestors = new ArrayList<>();
    while (parent != null) {
      ancestors.add(parent);
      parent = HaraLogicalPath.parent(parent);
    }
    java.util.Collections.reverse(ancestors);
    for (String ancestor : ancestors) {
      RemoteEntry value = optionalLstat(remote(ancestor), operation, path, target);
      if (value == null) {
        if (allowMissing) return;
        throw failure("not-found", "ancestor does not exist", operation, path, target, null, false, null);
      }
      if (value.type() == EntryType.SYMLINK) throw failure("outside-root", "symbolic-link traversal is not permitted", operation, path, target, "symlink", false, null);
      if (value.type() != EntryType.DIRECTORY) throw failure("not-directory", "ancestor is not a directory", operation, path, target, null, false, null);
    }
  }

  private void ensureParents(String path, boolean create, String operation) throws Exception {
    String parent = HaraLogicalPath.parent(path);
    if (parent == null) return;
    ArrayList<String> missing = new ArrayList<>();
    String cursor = parent;
    while (cursor != null) {
      RemoteEntry value = optionalLstat(remote(cursor), operation, path, null);
      if (value != null) {
        if (value.type() == EntryType.SYMLINK) throw failure("outside-root", "symbolic-link traversal is not permitted", operation, path, null, "symlink", false, null);
        if (value.type() != EntryType.DIRECTORY) throw failure("not-directory", "parent is not a directory", operation, path, null, null, false, null);
        break;
      }
      missing.add(cursor);
      cursor = HaraLogicalPath.parent(cursor);
    }
    if (!missing.isEmpty() && !create) throw failure("not-found", "parent directory does not exist", operation, path, null, null, false, null);
    java.util.Collections.reverse(missing);
    for (String directory : missing) client.mkdir(remote(directory), MutationContext.none());
  }

  private RemoteEntry optionalLstat(String remotePath, String operation, String path, String target) throws Exception {
    try {
      return client.lstat(remotePath);
    } catch (ClientFailure failure) {
      if ("not-found".equals(failure.code())) return null;
      throw mapFailure(operation, path, target, failure);
    }
  }

  private Entry entry(String logical, RemoteEntry value) {
    return new Entry(
        logical,
        HaraLogicalPath.fileName(logical),
        value.type(),
        value.size(),
        value.modifiedAt(),
        value.id(),
        value.revision(),
        value.capabilities(),
        value.extensions());
  }

  private Mutation mutation(String logical, RemoteEntry value) {
    return new Mutation(logical, value.revision(), null, value.extensions());
  }

  private void requireRegular(RemoteEntry value, String operation, String path, String target) {
    if (value.type() == EntryType.SYMLINK) throw unsupported(operation, path, target, "symlink-read");
    if (value.type() == EntryType.DIRECTORY) throw failure("is-directory", "path is a directory", operation, path, target, null, false, null);
    if (value.type() != EntryType.FILE) throw unsupported(operation, path, target, "non-regular-entry");
  }

  private String remote(String logical) {
    String canonical = normalise(logical);
    if ("/".equals(canonical)) return root;
    StringBuilder path = new StringBuilder(root);
    for (String segment : HaraLogicalPath.segments(canonical)) {
      if (!path.toString().endsWith("/")) path.append('/');
      path.append(encode(segment));
    }
    return path.toString();
  }

  private static String encode(String value) {
    StringBuilder output = new StringBuilder();
    for (byte octet : value.getBytes(java.nio.charset.StandardCharsets.UTF_8)) {
      int b = octet & 0xff;
      if ((b >= 'a' && b <= 'z') || (b >= 'A' && b <= 'Z') || (b >= '0' && b <= '9') || b == '-' || b == '_' || b == '.' || b == '~') {
        output.append((char) b);
      } else {
        output.append('%').append(String.format("%02X", b));
      }
    }
    return output.toString();
  }

  private static String rootUrl(String value) {
    String root = Objects.requireNonNull(value, "WebDAV root");
    if (!root.endsWith("/")) root += "/";
    return root;
  }

  private static String normalise(String path) {
    try {
      return HaraLogicalPath.normalise(path);
    } catch (HaraLogicalPath.Error error) {
      throw failure(error.code(), error.getMessage(), "path", path, null, null, false, error);
    }
  }

  private static int pageOffset(String token, int size) {
    if (token == null) return 0;
    try {
      int offset = Integer.parseInt(token);
      if (offset < 0 || offset > size) throw new NumberFormatException();
      return offset;
    } catch (NumberFormatException error) {
      throw failure("invalid-page-token", "invalid WebDAV page token", "entries", null, null, "page-token", false, error);
    }
  }

  private <T> CompletionStage<T> submit(CallContext context, String operation, String path, String target, Supplier<T> work) {
    Objects.requireNonNull(context, "filesystem call context");
    if (closed.get()) return CompletableFuture.failedFuture(FilesystemException.providerClosed("webdav", operation, path, target));
    CompletableFuture<T> future = new CompletableFuture<>();
    Pending pendingOperation = new Pending(future, operation, path, target);
    pending.add(pendingOperation);
    long timeoutNanos = Math.min(TimeUnit.MILLISECONDS.toNanos(operationTimeoutMillis), context.remainingNanos());
    ScheduledFuture<?> timeout =
        scheduler.schedule(
            () -> future.completeExceptionally(FilesystemException.timeout("webdav", operation, path, target)),
            Math.max(0L, timeoutNanos),
            TimeUnit.NANOSECONDS);
    AutoCloseable cancellation = context.onCancel(() -> future.completeExceptionally(FilesystemException.cancelled("webdav", operation, path, target)));
    ioExecutor.execute(
        () -> {
          try {
            context.check("webdav", operation, path, target);
            if (closed.get()) throw FilesystemException.providerClosed("webdav", operation, path, target);
            future.complete(work.get());
          } catch (Throwable error) {
            future.completeExceptionally(mapFailure(operation, path, target, error));
          }
        });
    future.whenComplete(
        (value, error) -> {
          pending.remove(pendingOperation);
          timeout.cancel(false);
          try {
            cancellation.close();
          } catch (Exception ignored) {
          }
        });
    return future;
  }

  private static RuntimeException mapFailure(String operation, String path, String target, Throwable error) {
    Throwable cause = error;
    while (cause instanceof CompletionException && cause.getCause() != null) cause = cause.getCause();
    if (cause instanceof FilesystemException filesystem) return filesystem;
    if (cause instanceof ClientFailure client) {
      return failure(client.code(), "WebDAV transport operation failed", operation, path, target, client.providerCode(), client.retryable(), client);
    }
    if (cause instanceof RuntimeException runtime) return runtime;
    return failure("io", "WebDAV operation failed", operation, path, target, cause.getClass().getSimpleName(), true, cause);
  }

  private static FilesystemException unsupported(String operation, String path, String target, String code) {
    return failure("unsupported", "WebDAV operation is unsupported", operation, path, target, code, false, null);
  }

  private static FilesystemException failure(
      String code,
      String message,
      String operation,
      String path,
      String target,
      String providerCode,
      boolean retryable,
      Throwable cause) {
    return new FilesystemException(code, message, "webdav", operation, path, target, providerCode, retryable, cause);
  }

  private static String requireText(String value, String label) {
    if (value == null || value.isBlank()) throw new IllegalArgumentException(label + " is required");
    return value;
  }
}
