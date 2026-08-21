package hara.truffle;

import java.net.URI;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.concurrent.atomic.AtomicBoolean;

final class WebDavFilesystemFixture implements AutoCloseable {
  static final String ORIGIN = "https://dav.example.test";
  static final String ROOT = "/files";
  static final URI ROOT_URI = URI.create(ORIGIN + ROOT);
  final MemoryClient client = new MemoryClient();

  WebDavFilesystemFixture() {
    client.directory(ROOT_URI);
    client.directory(ROOT_URI.resolve("files/docs").normalize());
    client.file(ROOT_URI.resolve("files/README.md").normalize(), "hello".getBytes(StandardCharsets.UTF_8));
    client.file(ROOT_URI.resolve("files/docs/a.bin").normalize(), new byte[] {1});
    client.file(ROOT_URI.resolve("files/docs/b.bin").normalize(), new byte[] {2});
  }

  @Override
  public void close() throws Exception {
    client.close();
  }

  static final class MemoryClient implements WebDavFilesystem.Client {
    private record Value(
        IFilesystem.EntryType type, byte[] bytes, long modifiedAt, String revision) {}

    private final Map<URI, Value> values = new HashMap<>();
    private final AtomicBoolean closed = new AtomicBoolean();
    private final Set<IFilesystem.Capability> capabilities =
        Set.of(
            IFilesystem.Capability.READ,
            IFilesystem.Capability.WRITE,
            IFilesystem.Capability.ENTRIES,
            IFilesystem.Capability.MKDIR,
            IFilesystem.Capability.DELETE,
            IFilesystem.Capability.COPY,
            IFilesystem.Capability.MOVE,
            IFilesystem.Capability.REVISION_CHECK);
    boolean authenticated = true;
    int sequence = 1;
    WebDavFilesystem.ResourcePage overridePage;

    @Override
    public boolean authenticated() {
      return authenticated;
    }

    @Override
    public Set<IFilesystem.Capability> capabilities() {
      return capabilities;
    }

    @Override
    public synchronized WebDavFilesystem.Resource stat(URI href) throws Exception {
      requireOpen();
      Value value = values.get(canonical(href));
      if (value == null) throw missing();
      return resource(canonical(href), value);
    }

    @Override
    public synchronized byte[] read(URI href, long maxBytes) throws Exception {
      requireOpen();
      Value value = values.get(canonical(href));
      if (value == null) throw missing();
      if (value.type() != IFilesystem.EntryType.FILE) {
        throw new WebDavFilesystem.ClientFailure("is-directory", "collection", false);
      }
      if (value.bytes().length > maxBytes) {
        throw new WebDavFilesystem.ClientFailure("quota-exceeded", "fixture-transfer-limit", false);
      }
      return value.bytes().clone();
    }

    @Override
    public synchronized WebDavFilesystem.Resource write(
        URI href,
        byte[] bytes,
        IFilesystem.WriteMode mode,
        String expectedRevision,
        boolean createOnly)
        throws Exception {
      requireOpen();
      URI key = canonical(href);
      Value existing = values.get(key);
      if (createOnly && existing != null) {
        throw new WebDavFilesystem.ClientFailure("already-exists", "412", false);
      }
      checkExpected(existing, expectedRevision);
      Value value = value(IFilesystem.EntryType.FILE, bytes);
      values.put(key, value);
      return resource(key, value);
    }

    @Override
    public synchronized WebDavFilesystem.ResourcePage entries(
        URI collection, String continuationToken, int limit) throws Exception {
      requireOpen();
      if (overridePage != null) return overridePage;
      URI parent = canonical(collection);
      Value parentValue = values.get(parent);
      if (parentValue == null) throw missing();
      if (parentValue.type() != IFilesystem.EntryType.DIRECTORY) {
        throw new WebDavFilesystem.ClientFailure("not-directory", "not-collection", false);
      }
      ArrayList<WebDavFilesystem.Resource> children = new ArrayList<>();
      String prefix = path(parent);
      if (!prefix.endsWith("/")) prefix += "/";
      for (Map.Entry<URI, Value> entry : values.entrySet()) {
        if (!sameOrigin(parent, entry.getKey())) continue;
        String candidate = path(entry.getKey());
        if (!candidate.startsWith(prefix)) continue;
        String remainder = candidate.substring(prefix.length());
        if (remainder.isEmpty() || remainder.contains("/")) continue;
        children.add(resource(entry.getKey(), entry.getValue()));
      }
      children.sort(Comparator.comparing(WebDavFilesystem.Resource::name));
      int offset = continuationToken == null ? 0 : Integer.parseInt(continuationToken);
      int end = Math.min(children.size(), offset + limit);
      return new WebDavFilesystem.ResourcePage(
          children.subList(offset, end), end < children.size() ? Integer.toString(end) : null);
    }

    @Override
    public synchronized WebDavFilesystem.Resource mkdir(URI href, boolean existsOk) throws Exception {
      requireOpen();
      URI key = canonical(href);
      Value existing = values.get(key);
      if (existing != null) {
        if (existsOk && existing.type() == IFilesystem.EntryType.DIRECTORY) return resource(key, existing);
        throw new WebDavFilesystem.ClientFailure("already-exists", "405", false);
      }
      Value value = value(IFilesystem.EntryType.DIRECTORY, null);
      values.put(key, value);
      return resource(key, value);
    }

    @Override
    public synchronized void delete(URI href, String expectedRevision) throws Exception {
      requireOpen();
      URI key = canonical(href);
      Value existing = values.get(key);
      if (existing == null) throw missing();
      checkExpected(existing, expectedRevision);
      String rawPrefix = path(key);
      String childPrefix = rawPrefix.endsWith("/") ? rawPrefix : rawPrefix + "/";
      values.keySet().removeIf(
          candidate -> sameOrigin(key, candidate) && path(candidate).startsWith(childPrefix));
      values.remove(key);
    }

    @Override
    public synchronized WebDavFilesystem.Resource copy(
        URI source,
        URI target,
        boolean replace,
        String expectedSourceRevision,
        String expectedTargetRevision)
        throws Exception {
      requireOpen();
      URI sourceKey = canonical(source);
      URI targetKey = canonical(target);
      Value sourceValue = values.get(sourceKey);
      if (sourceValue == null) throw missing();
      checkExpected(sourceValue, expectedSourceRevision);
      Value targetValue = values.get(targetKey);
      if (!replace && targetValue != null) {
        throw new WebDavFilesystem.ClientFailure("already-exists", "412", false);
      }
      checkExpected(targetValue, expectedTargetRevision);
      Value copied = value(sourceValue.type(), sourceValue.bytes());
      values.put(targetKey, copied);
      return resource(targetKey, copied);
    }

    @Override
    public synchronized WebDavFilesystem.Resource move(
        URI source,
        URI target,
        boolean replace,
        boolean atomic,
        String expectedSourceRevision,
        String expectedTargetRevision)
        throws Exception {
      WebDavFilesystem.Resource copied =
          copy(source, target, replace, expectedSourceRevision, expectedTargetRevision);
      values.remove(canonical(source));
      return copied;
    }

    @Override
    public void close() {
      closed.set(true);
    }

    synchronized void file(URI href, byte[] bytes) {
      values.put(canonical(href), value(IFilesystem.EntryType.FILE, bytes));
    }

    synchronized void directory(URI href) {
      values.put(canonical(href), value(IFilesystem.EntryType.DIRECTORY, null));
    }

    synchronized boolean exists(URI href) {
      return values.containsKey(canonical(href));
    }

    synchronized byte[] bytes(URI href) {
      Value value = values.get(canonical(href));
      return value == null || value.bytes() == null ? null : value.bytes().clone();
    }

    private Value value(IFilesystem.EntryType type, byte[] bytes) {
      int current = sequence++;
      return new Value(type, bytes == null ? null : bytes.clone(), current, "etag-" + current);
    }

    private WebDavFilesystem.Resource resource(URI href, Value value) {
      String name = ROOT_URI.equals(href) ? "" : fileName(path(href));
      return new WebDavFilesystem.Resource(
          href.toString(),
          name,
          value.type(),
          value.type() == IFilesystem.EntryType.FILE ? (long) value.bytes().length : null,
          value.modifiedAt(),
          value.revision(),
          new IFilesystem.Capabilities(capabilities),
          Map.of("provider/status", "fixture"));
    }

    private void checkExpected(Value value, String expected) throws Exception {
      if (expected == null) return;
      if (value == null || !expected.equals(value.revision())) {
        throw new WebDavFilesystem.ClientFailure("conflict", "412", false);
      }
    }

    private void requireOpen() throws Exception {
      if (closed.get()) throw new WebDavFilesystem.ClientFailure("provider-closed", "closed", false);
    }

    private static WebDavFilesystem.ClientFailure missing() {
      return new WebDavFilesystem.ClientFailure("not-found", "404", false);
    }

    private static URI canonical(URI href) {
      String text = href.toString();
      while (text.length() > ORIGIN.length() + 1 && text.endsWith("/")) {
        text = text.substring(0, text.length() - 1);
      }
      return URI.create(text);
    }

    private static String path(URI href) {
      return href.getRawPath();
    }

    private static String fileName(String path) {
      int slash = path.lastIndexOf('/');
      return slash < 0 ? path : path.substring(slash + 1);
    }

    private static boolean sameOrigin(URI left, URI right) {
      return left.getScheme().equalsIgnoreCase(right.getScheme())
          && left.getHost().equals(right.getHost())
          && left.getPort() == right.getPort();
    }
  }
}
