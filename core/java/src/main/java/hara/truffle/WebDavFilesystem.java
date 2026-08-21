package hara.truffle;

import java.net.URI;
import java.net.URISyntaxException;
import java.net.URLDecoder;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
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

/** Provider-neutral WebDAV mount over a trusted authenticated DAV transport capability. */
final class WebDavFilesystem implements IFilesystem {
  record Resource(
      String href,
      String name,
      EntryType type,
      Long size,
      Long modifiedAt,
      String revision,
      Capabilities capabilities,
      Map<String, Object> extensions) {
    Resource {
      href = requireText(href, "WebDAV href");
      if (name == null || name.contains("/") || ".".equals(name) || "..".equals(name)) {
        throw new IllegalArgumentException("invalid WebDAV entry name");
      }
      type = Objects.requireNonNull(type, "WebDAV entry type");
      if (size != null && size < 0) throw new IllegalArgumentException("negative WebDAV entry size");
      capabilities = capabilities == null ? new Capabilities(Set.of()) : capabilities;
      extensions = extensions == null || extensions.isEmpty() ? Map.of() : Map.copyOf(extensions);
    }
  }

  record ResourcePage(List<Resource> resources, String nextToken) {
    ResourcePage {
      resources = List.copyOf(Objects.requireNonNull(resources, "WebDAV resources"));
    }
  }

  /** Trusted DAV client. Authentication, TLS policy and redirect handling remain host-owned. */
  interface Client extends AutoCloseable {
    boolean authenticated();

    Set<Capability> capabilities();

    Resource stat(URI href) throws Exception;

    byte[] read(URI href, long maxBytes) throws Exception;

    Resource write(
        URI href, byte[] bytes, WriteMode mode, String expectedRevision, boolean createOnly)
        throws Exception;

    ResourcePage entries(URI collection, String continuationToken, int limit) throws Exception;

    Resource mkdir(URI href, boolean existsOk) throws Exception;

    void delete(URI href, String expectedRevision) throws Exception;

    Resource copy(
        URI source,
        URI target,
        boolean replace,
        String expectedSourceRevision,
        String expectedTargetRevision)
        throws Exception;

    Resource move(
        URI source,
        URI target,
        boolean replace,
        boolean atomic,
        String expectedSourceRevision,
        String expectedTargetRevision)
        throws Exception;

    @Override
    void close() throws Exception;
  }

  static final class ClientFailure extends Exception {
    private static final long serialVersionUID = 1L;
    private final String code;
    private final String providerCode;
    private final boolean retryable;

    ClientFailure(String code, String providerCode, boolean retryable) {
      super("WebDAV transport operation failed");
      if (code == null || !code.matches("[a-z][a-z0-9-]*")) {
        throw new IllegalArgumentException("invalid WebDAV failure code");
      }
      this.code = code;
      this.providerCode = providerCode;
      this.retryable = retryable;
    }

    String code() {
      return code;
    }

    String providerCode() {
      return providerCode;
    }

    boolean retryable() {
      return retryable;
    }
  }

  static final class Factory implements IFilesystemFactory {
    private static final Set<String> ALLOWED =
        Set.of(
            "credential-ref",
            "origin",
            "root",
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
      requireText(configuration.get("credential-ref"), "WebDAV credential-ref");
      mountedRoot(
          requireText(configuration.get("origin"), "WebDAV origin"),
          requireText(configuration.get("root"), "WebDAV root"));
      Object readOnly = configuration.get("read-only?");
      if (readOnly != null && !(readOnly instanceof Boolean)) {
        throw new IllegalArgumentException("WebDAV read-only? must be a boolean");
      }
      Object display = configuration.get("display");
      if (display != null) requireText(display, "WebDAV display");
      positiveLong(configuration, "operation-timeout-ms", 30_000L);
      positiveLong(configuration, "max-transfer-bytes", 16L * 1024L * 1024L);
    }

    @Override
    public CompletionStage<IFilesystem> open(OpenContext context, Map<String, ?> configuration) {
      Objects.requireNonNull(context, "filesystem open context");
      validate(configuration);
      Object resolved = context.credentials().resolve((String) configuration.get("credential-ref"));
      if (!(resolved instanceof Client client)) {
        return CompletableFuture.failedFuture(
            new IllegalArgumentException(
                "WebDAV credential reference did not resolve to a trusted DAV client"));
      }
      if (!client.authenticated()) {
        return CompletableFuture.failedFuture(
            failure(
                "authentication-failed",
                "WebDAV client is not authenticated",
                "open",
                null,
                null,
                "client-unauthenticated",
                false,
                null));
      }
      URI root =
          mountedRoot((String) configuration.get("origin"), (String) configuration.get("root"));
      WebDavFilesystem filesystem =
          new WebDavFilesystem(
              client,
              root,
              configuration.get("display") instanceof String value ? value : "WebDAV filesystem",
              Boolean.TRUE.equals(configuration.get("read-only?")),
              positiveLong(configuration, "operation-timeout-ms", 30_000L),
              positiveLong(configuration, "max-transfer-bytes", 16L * 1024L * 1024L),
              context.ioExecutor(),
              context.scheduler());
      return filesystem.submit(
          IFilesystem.CallContext.create(),
          "open",
          "/",
          null,
          () -> {
            Resource resource = filesystem.checked(filesystem.client.stat(root), "/", "open", null);
            if (resource.type() != EntryType.DIRECTORY) {
              throw failure(
                  "not-directory",
                  "WebDAV mounted root is not a collection",
                  "open",
                  "/",
                  null,
                  "root-not-collection",
                  false,
                  null);
            }
            return (IFilesystem) filesystem;
          });
    }

    private static long positiveLong(Map<String, ?> values, String key, long fallback) {
      Object value = values.get(key);
      if (value == null) return fallback;
      if (!(value instanceof Number number) || number.longValue() <= 0) {
        throw new IllegalArgumentException("WebDAV " + key + " must be positive");
      }
      return number.longValue();
    }
  }

  private record Pending(CompletableFuture<?> future, String operation, String path, String target) {}

  private final Client client;
  private final URI root;
  private final String display;
  private final boolean readOnly;
  private final long operationTimeoutMillis;
  private final long maxTransferBytes;
  private final Executor ioExecutor;
  private final ScheduledExecutorService scheduler;
  private final Capabilities capabilities;
  private final AtomicBoolean closed = new AtomicBoolean();
  private final Set<Pending> pending = ConcurrentHashMap.newKeySet();

  WebDavFilesystem(
      Client client,
      URI root,
      String display,
      boolean readOnly,
      long operationTimeoutMillis,
      long maxTransferBytes,
      Executor ioExecutor,
      ScheduledExecutorService scheduler) {
    this.client = Objects.requireNonNull(client, "WebDAV client");
    this.root = Objects.requireNonNull(root, "WebDAV root");
    this.display = requireText(display, "WebDAV display");
    this.readOnly = readOnly;
    this.operationTimeoutMillis = operationTimeoutMillis;
    this.maxTransferBytes = maxTransferBytes;
    this.ioExecutor = Objects.requireNonNull(ioExecutor, "filesystem I/O executor");
    this.scheduler = Objects.requireNonNull(scheduler, "filesystem scheduler");
    this.capabilities = new Capabilities(advertised(client.capabilities(), readOnly));
  }

  @Override
  public Descriptor descriptor() {
    return new Descriptor(
        "webdav",
        display,
        readOnly,
        capabilities,
        null,
        Map.of("provider/root-scoped?", true, "provider/hierarchical?", true));
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
          return entry(checked(client.stat(href(logical)), logical, "stat", null), logical);
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
          Resource resource = checked(client.stat(href(logical)), logical, "read", null);
          if (resource.type() == EntryType.DIRECTORY) {
            throw failure("is-directory", "path is a DAV collection", "read", logical, null, null, false, null);
          }
          if (resource.type() != EntryType.FILE) {
            throw unsupported("read", logical, null, "non-file-resource");
          }
          if (resource.size() != null && resource.size() > maxTransferBytes) {
            throw transferLimit("read", logical, null);
          }
          byte[] bytes = client.read(href(logical), maxTransferBytes);
          if (bytes.length > maxTransferBytes) throw transferLimit("read", logical, null);
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
    if (options.parents()) {
      return CompletableFuture.failedFuture(unsupported("write", logical, null, "parent-creation-unavailable"));
    }
    if (options.mode() == WriteMode.APPEND) {
      return CompletableFuture.failedFuture(unsupported("write", logical, null, "append-unavailable"));
    }
    return submit(
        context,
        "write",
        logical,
        null,
        () -> {
          requireWritable(Capability.WRITE, "write", logical, null);
          requireRevision(mutation, "write", logical, null);
          if ("/".equals(logical)) {
            throw failure("is-directory", "mounted root is a DAV collection", "write", logical, null, null, false, null);
          }
          if (copy.length > maxTransferBytes) throw transferLimit("write", logical, null);
          Resource resource =
              checked(
                  client.write(
                      href(logical),
                      copy,
                      options.mode(),
                      mutation.expectedRevision(),
                      options.mode() == WriteMode.CREATE),
                  logical,
                  "write",
                  null);
          return mutation(logical, resource);
        });
  }

  @Override
  public CompletionStage<EntryPage> entriesPage(
      CallContext context, String path, PageRequest request) {
    String logical = normalise(path);
    Objects.requireNonNull(request, "filesystem page request");
    return submit(
        context,
        "entries",
        logical,
        null,
        () -> {
          require(Capability.ENTRIES, "entries", logical, null);
          Resource collection = checked(client.stat(href(logical)), logical, "entries", null);
          if (collection.type() != EntryType.DIRECTORY) {
            throw failure("not-directory", "path is not a DAV collection", "entries", logical, null, null, false, null);
          }
          ResourcePage page = client.entries(href(logical), request.token(), request.limit());
          ArrayList<Entry> values = new ArrayList<>();
          HashSet<String> names = new HashSet<>();
          for (Resource resource : page.resources()) {
            String name = resource.name();
            if (name == null || name.isBlank()) {
              throw failure("invalid-entry", "DAV child has no name", "entries", logical, null, "missing-name", false, null);
            }
            String child = HaraLogicalPath.join(logical, name);
            checked(resource, child, "entries", null);
            if (!sameHref(href(child), URI.create(resource.href()))) {
              throw failure(
                  "outside-root",
                  "DAV child href does not match its canonical child path",
                  "entries",
                  child,
                  null,
                  "ambiguous-href",
                  false,
                  null);
            }
            if (!names.add(name)) {
              throw failure("ambiguous-path", "duplicate DAV child name", "entries", child, null, "duplicate-name", false, null);
            }
            values.add(entry(resource, child));
          }
          return new EntryPage(values, page.nextToken());
        });
  }

  @Override
  public CompletionStage<Mutation> mkdir(
      CallContext context, String path, MkdirOptions options, MutationContext mutation) {
    String logical = normalise(path);
    Objects.requireNonNull(options, "mkdir options");
    Objects.requireNonNull(mutation, "mutation context");
    if (options.parents()) {
      return CompletableFuture.failedFuture(unsupported("mkdir", logical, null, "parent-creation-unavailable"));
    }
    return submit(
        context,
        "mkdir",
        logical,
        null,
        () -> {
          requireWritable(Capability.MKDIR, "mkdir", logical, null);
          if (mutation.required()) throw FilesystemException.unsupportedRevision("webdav", "mkdir", logical, null);
          if ("/".equals(logical) && options.existsOk()) return Mutation.path("/");
          Resource created = checked(client.mkdir(href(logical), options.existsOk()), logical, "mkdir", null);
          if (created.type() != EntryType.DIRECTORY) {
            throw failure("io", "MKCOL did not create a DAV collection", "mkdir", logical, null, "not-collection", false, null);
          }
          return mutation(logical, created);
        });
  }

  @Override
  public CompletionStage<Mutation> delete(
      CallContext context, String path, DeleteOptions options, MutationContext mutation) {
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
          requireRevision(mutation, "delete", logical, null);
          if ("/".equals(logical)) {
            throw failure("denied", "cannot delete mounted DAV root", "delete", logical, null, null, false, null);
          }
          try {
            Resource current = checked(client.stat(href(logical)), logical, "delete", null);
            checkExpected(current, mutation.expectedRevision(), "delete", logical, null);
            client.delete(href(logical), mutation.expectedRevision());
          } catch (ClientFailure error) {
            if (options.missingOk() && "not-found".equals(error.code())) return Mutation.path(logical);
            throw error;
          }
          return Mutation.path(logical);
        });
  }

  @Override
  public CompletionStage<Mutation> copy(
      CallContext context,
      String source,
      String target,
      CopyOptions options,
      MutationContext mutation) {
    String sourceLogical = normalise(source);
    String targetLogical = normalise(target);
    Objects.requireNonNull(options, "copy options");
    Objects.requireNonNull(mutation, "mutation context");
    if (options.parents()) {
      return CompletableFuture.failedFuture(unsupported("copy", sourceLogical, targetLogical, "parent-creation-unavailable"));
    }
    if (options.preserveModified() && !capabilities.contains(Capability.PRESERVE_MODIFIED)) {
      return CompletableFuture.failedFuture(unsupported("copy", sourceLogical, targetLogical, "preserve-modified-unavailable"));
    }
    return submit(
        context,
        "copy",
        sourceLogical,
        targetLogical,
        () -> {
          requireWritable(Capability.COPY, "copy", sourceLogical, targetLogical);
          requireRevision(mutation, "copy", sourceLogical, targetLogical);
          Resource sourceValue = checked(client.stat(href(sourceLogical)), sourceLogical, "copy", targetLogical);
          checkExpected(sourceValue, mutation.expectedRevision(), "copy", sourceLogical, targetLogical);
          Resource copied =
              checked(
                  client.copy(
                      href(sourceLogical),
                      href(targetLogical),
                      options.replace(),
                      mutation.expectedRevision(),
                      mutation.expectedTargetRevision()),
                  targetLogical,
                  "copy",
                  sourceLogical);
          return mutation(targetLogical, copied);
        });
  }

  @Override
  public CompletionStage<Mutation> move(
      CallContext context,
      String source,
      String target,
      MoveOptions options,
      MutationContext mutation) {
    String sourceLogical = normalise(source);
    String targetLogical = normalise(target);
    Objects.requireNonNull(options, "move options");
    Objects.requireNonNull(mutation, "mutation context");
    if (options.parents()) {
      return CompletableFuture.failedFuture(unsupported("move", sourceLogical, targetLogical, "parent-creation-unavailable"));
    }
    if (options.atomic() && !capabilities.contains(Capability.ATOMIC_MOVE)) {
      return CompletableFuture.failedFuture(unsupported("move", sourceLogical, targetLogical, "atomic-move-unavailable"));
    }
    return submit(
        context,
        "move",
        sourceLogical,
        targetLogical,
        () -> {
          requireWritable(Capability.MOVE, "move", sourceLogical, targetLogical);
          requireRevision(mutation, "move", sourceLogical, targetLogical);
          Resource sourceValue = checked(client.stat(href(sourceLogical)), sourceLogical, "move", targetLogical);
          checkExpected(sourceValue, mutation.expectedRevision(), "move", sourceLogical, targetLogical);
          Resource moved =
              checked(
                  client.move(
                      href(sourceLogical),
                      href(targetLogical),
                      options.replace(),
                      options.atomic(),
                      mutation.expectedRevision(),
                      mutation.expectedTargetRevision()),
                  targetLogical,
                  "move",
                  sourceLogical);
          return mutation(targetLogical, moved);
        });
  }

  @Override
  public CompletionStage<Void> close(CallContext context) {
    Objects.requireNonNull(context, "filesystem call context");
    if (!closed.compareAndSet(false, true)) return CompletableFuture.completedFuture(null);
    for (Pending operation : pending) {
      operation.future().completeExceptionally(
          FilesystemException.providerClosed(
              "webdav", operation.operation(), operation.path(), operation.target()));
    }
    pending.clear();
    CompletableFuture<Void> result = new CompletableFuture<>();
    ioExecutor.execute(
        () -> {
          try {
            client.close();
            result.complete(null);
          } catch (Throwable error) {
            result.completeExceptionally(mapFailure(error, "close", null, null));
          }
        });
    return result;
  }

  private Resource checked(Resource resource, String logical, String operation, String target) {
    Objects.requireNonNull(resource, "WebDAV resource");
    URI value;
    try {
      value = URI.create(resource.href());
    } catch (IllegalArgumentException error) {
      throw failure("outside-root", "invalid DAV href", operation, logical, target, "invalid-href", false, error);
    }
    if (!inside(root, value)) {
      throw failure("outside-root", "DAV href escapes mounted authority", operation, logical, target, "href-outside-root", false, null);
    }
    if (resource.type() == EntryType.SYMLINK || resource.type() == EntryType.OTHER) {
      throw unsupported(operation, logical, target, "unsupported-resource-type");
    }
    return resource;
  }

  private Entry entry(Resource resource, String logical) {
    return new Entry(
        logical,
        "/".equals(logical) ? "" : HaraLogicalPath.fileName(logical),
        resource.type(),
        resource.type() == EntryType.FILE ? resource.size() : null,
        resource.modifiedAt(),
        null,
        resource.revision(),
        resource.capabilities(),
        resource.extensions());
  }

  private Mutation mutation(String logical, Resource resource) {
    return new Mutation(logical, resource.revision(), null, Map.of());
  }

  private static Set<Capability> advertised(Set<Capability> transport, boolean readOnly) {
    HashSet<Capability> values = new HashSet<>();
    for (Capability value : List.of(Capability.READ, Capability.ENTRIES)) {
      if (transport.contains(value)) values.add(value);
    }
    if (!readOnly) {
      for (Capability value :
          List.of(
              Capability.WRITE,
              Capability.MKDIR,
              Capability.DELETE,
              Capability.COPY,
              Capability.MOVE,
              Capability.ATOMIC_MOVE,
              Capability.PRESERVE_MODIFIED,
              Capability.REVISION_CHECK)) {
        if (transport.contains(value)) values.add(value);
      }
    }
    return Set.copyOf(values);
  }

  private void require(Capability capability, String operation, String path, String target) {
    if (!capabilities.contains(capability)) {
      throw unsupported(operation, path, target, capability.keyword() + "-unavailable");
    }
  }

  private void requireWritable(Capability capability, String operation, String path, String target) {
    if (readOnly) {
      throw failure("permission-denied", "WebDAV mount is read-only", operation, path, target, "read-only", false, null);
    }
    require(capability, operation, path, target);
  }

  private void requireRevision(MutationContext mutation, String operation, String path, String target) {
    if (mutation.required() && !capabilities.contains(Capability.REVISION_CHECK)) {
      throw FilesystemException.unsupportedRevision("webdav", operation, path, target);
    }
  }

  private static void checkExpected(
      Resource resource, String expected, String operation, String path, String target) {
    if (expected == null) return;
    if (resource.revision() == null || !expected.equals(resource.revision())) {
      throw failure("conflict", "WebDAV revision does not match", operation, path, target, "revision-mismatch", false, null);
    }
  }

  private URI href(String logical) {
    String normal = normalise(logical);
    if ("/".equals(normal)) return root;
    StringBuilder relative = new StringBuilder();
    for (String segment : normal.substring(1).split("/")) {
      if (!relative.isEmpty()) relative.append('/');
      relative.append(encodeSegment(segment));
    }
    String base = root.toString();
    if (!base.endsWith("/")) base += "/";
    URI value = URI.create(base + relative);
    if (!inside(root, value)) throw new HaraLogicalPath.Error("outside-root", "path escapes WebDAV root");
    return value;
  }

  private <T> CompletionStage<T> submit(
      CallContext context,
      String operation,
      String path,
      String target,
      ThrowingSupplier<T> body) {
    Objects.requireNonNull(context, "filesystem call context");
    if (closed.get()) {
      return CompletableFuture.failedFuture(FilesystemException.providerClosed("webdav", operation, path, target));
    }
    try {
      context.check("webdav", operation, path, target);
    } catch (RuntimeException error) {
      return CompletableFuture.failedFuture(error);
    }
    CompletableFuture<T> result = new CompletableFuture<>();
    Pending tracked = new Pending(result, operation, path, target);
    pending.add(tracked);
    long timeoutNanos = TimeUnit.MILLISECONDS.toNanos(operationTimeoutMillis);
    if (context.hasDeadline()) timeoutNanos = Math.min(timeoutNanos, context.remainingNanos());
    ScheduledFuture<?> timeout =
        scheduler.schedule(
            () -> result.completeExceptionally(FilesystemException.timeout("webdav", operation, path, target)),
            Math.max(0L, timeoutNanos),
            TimeUnit.NANOSECONDS);
    AutoCloseable cancellation =
        context.onCancel(
            () -> result.completeExceptionally(FilesystemException.cancelled("webdav", operation, path, target)));
    result.whenComplete(
        (ignored, error) -> {
          timeout.cancel(false);
          pending.remove(tracked);
          try {
            cancellation.close();
          } catch (Exception ignoredClose) {
            // Hook removal is best-effort after settlement.
          }
        });
    ioExecutor.execute(
        () -> {
          if (result.isDone()) return;
          try {
            context.check("webdav", operation, path, target);
            result.complete(body.get());
          } catch (Throwable error) {
            result.completeExceptionally(mapFailure(error, operation, path, target));
          }
        });
    return result;
  }

  private static FilesystemException mapFailure(
      Throwable error, String operation, String path, String target) {
    Throwable current = unwrap(error);
    if (current instanceof FilesystemException filesystem) return filesystem;
    if (current instanceof ClientFailure failure) {
      return failure(
          failure.code(),
          "WebDAV transport operation failed",
          operation,
          path,
          target,
          failure.providerCode(),
          failure.retryable(),
          failure);
    }
    if (current instanceof HaraLogicalPath.Error logical) {
      return failure(logical.code(), logical.getMessage(), operation, path, target, null, false, logical);
    }
    return failure("io", "WebDAV filesystem operation failed", operation, path, target, "transport-error", true, current);
  }

  private static Throwable unwrap(Throwable error) {
    Throwable current = error;
    while ((current instanceof CompletionException
            || current instanceof java.util.concurrent.ExecutionException)
        && current.getCause() != null) {
      current = current.getCause();
    }
    return current;
  }

  private static FilesystemException transferLimit(String operation, String path, String target) {
    return failure("quota-exceeded", "WebDAV transfer exceeds configured limit", operation, path, target, "transfer-limit", false, null);
  }

  private static FilesystemException unsupported(
      String operation, String path, String target, String providerCode) {
    return failure("unsupported", "WebDAV provider does not support the requested operation", operation, path, target, providerCode, false, null);
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

  static URI mountedRoot(String originText, String rootText) {
    URI origin;
    try {
      origin = URI.create(originText);
    } catch (IllegalArgumentException error) {
      throw new IllegalArgumentException("WebDAV origin must be a valid HTTPS origin", error);
    }
    if (!"https".equalsIgnoreCase(origin.getScheme())
        || origin.getHost() == null
        || origin.getUserInfo() != null
        || origin.getQuery() != null
        || origin.getFragment() != null
        || !(origin.getRawPath() == null || origin.getRawPath().isEmpty() || "/".equals(origin.getRawPath()))) {
      throw new IllegalArgumentException("WebDAV origin must contain only HTTPS scheme and authority");
    }
    if (!rootText.startsWith("/") || rootText.contains("\\") || rootText.indexOf('\0') >= 0) {
      throw new IllegalArgumentException("WebDAV root must be an absolute collection path");
    }
    URI root = origin.resolve(rootText);
    if (!insideOrigin(origin, root) || hasDotSegment(root.getRawPath())) {
      throw new IllegalArgumentException("WebDAV root escapes configured origin");
    }
    String raw = root.getRawPath();
    if (raw == null || raw.isBlank()) raw = "/";
    while (raw.length() > 1 && raw.endsWith("/")) raw = raw.substring(0, raw.length() - 1);
    try {
      return new URI(root.getScheme(), null, root.getHost(), root.getPort(), raw, null, null);
    } catch (URISyntaxException error) {
      throw new IllegalArgumentException("invalid WebDAV root", error);
    }
  }

  private static boolean inside(URI root, URI value) {
    if (!insideOrigin(root, value)
        || value.getUserInfo() != null
        || value.getQuery() != null
        || value.getFragment() != null
        || hasDotSegment(value.getRawPath())) {
      return false;
    }
    String rootPath = canonicalRawPath(root.getRawPath());
    String path = canonicalRawPath(value.getRawPath());
    return path.equals(rootPath) || path.startsWith("/".equals(rootPath) ? "/" : rootPath + "/");
  }

  private static boolean insideOrigin(URI origin, URI value) {
    return "https".equalsIgnoreCase(value.getScheme())
        && Objects.equals(origin.getHost(), value.getHost())
        && effectivePort(origin) == effectivePort(value);
  }

  private static int effectivePort(URI value) {
    return value.getPort() < 0 ? 443 : value.getPort();
  }

  private static String canonicalRawPath(String raw) {
    if (raw == null || raw.isEmpty()) return "/";
    String value = raw;
    while (value.length() > 1 && value.endsWith("/")) value = value.substring(0, value.length() - 1);
    return value;
  }

  private static boolean hasDotSegment(String rawPath) {
    if (rawPath == null) return false;
    for (String raw : rawPath.split("/", -1)) {
      String decoded;
      try {
        decoded = URLDecoder.decode(raw.replace("+", "%2B"), StandardCharsets.UTF_8);
      } catch (IllegalArgumentException error) {
        return true;
      }
      if (".".equals(decoded) || "..".equals(decoded)) return true;
    }
    return false;
  }

  private static boolean sameHref(URI expected, URI actual) {
    return inside(expected, actual)
        && canonicalRawPath(expected.getRawPath()).equals(canonicalRawPath(actual.getRawPath()));
  }

  private static String encodeSegment(String value) {
    try {
      String raw = new URI(null, null, "/" + value, null).getRawPath();
      return raw.substring(1);
    } catch (URISyntaxException error) {
      throw new IllegalArgumentException("invalid WebDAV path segment", error);
    }
  }

  private static String normalise(String path) {
    return HaraLogicalPath.normalise(path);
  }

  private static String requireText(Object value, String label) {
    if (!(value instanceof String text) || text.isBlank()) {
      throw new IllegalArgumentException(label + " is required");
    }
    return text;
  }

  @FunctionalInterface
  private interface ThrowingSupplier<T> {
    T get() throws Exception;
  }
}
