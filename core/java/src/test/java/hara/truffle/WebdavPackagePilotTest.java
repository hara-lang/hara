package hara.truffle;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;

import java.nio.file.Files;
import java.nio.file.Path;
import java.security.MessageDigest;
import java.util.HexFormat;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.concurrent.Executors;
import java.util.concurrent.ScheduledExecutorService;
import org.junit.Test;

public class WebdavPackagePilotTest {
  @Test
  public void prebuiltProviderJarRegistersAndOpensThroughTrustedLoader() throws Exception {
    String configured = System.getProperty("hara.webdav.provider.jar");
    if (configured == null || configured.isBlank()) {
      throw new AssertionError("hara.webdav.provider.jar must point at the prebuilt provider JAR");
    }
    Path artifact = Path.of(configured).toAbsolutePath().normalize();
    assertTrue(Files.isRegularFile(artifact));
    String digest =
        "sha256:"
            + HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(artifact)));

    FilesystemProviderRegistry registry = new FilesystemProviderRegistry();
    JvmPackageLoader.Selection selection =
        new JvmPackageLoader.Selection(
            "hara:hara/filesystem-webdav",
            artifact,
            digest,
            JvmPackageProvider.ABI,
            "hara.provider.webdav.WebdavPackageProvider",
            Set.of());

    try (JvmPackageLoader.LoadedProvider loaded = JvmPackageLoader.load(selection, registry)) {
      assertEquals("hara:hara/filesystem-webdav", loaded.identity());
      assertTrue(registry.contains("webdav"));

      ScheduledExecutorService scheduler = Executors.newSingleThreadScheduledExecutor();
      try {
        WebdavFilesystem.Client client = new FixtureClient();
        IFilesystem filesystem =
            registry
                .open(
                    "webdav",
                    new IFilesystemFactory.OpenContext(Runnable::run, scheduler, ignored -> client),
                    Map.of(
                        "credential-ref", "fixture",
                        "root-url", "https://example.invalid/dav/",
                        "read-only?", true))
                .toCompletableFuture()
                .join();
        assertEquals("webdav", filesystem.descriptor().kind());
        assertTrue(filesystem.descriptor().readOnly());
        filesystem.close(IFilesystem.CallContext.create()).toCompletableFuture().join();
      } finally {
        scheduler.shutdownNow();
      }
    }
  }

  private static final class FixtureClient implements WebdavFilesystem.Client {
    @Override
    public boolean authenticated() {
      return true;
    }

    @Override
    public boolean transportVerified() {
      return true;
    }

    @Override
    public Set<IFilesystem.Capability> capabilities() {
      return Set.of(IFilesystem.Capability.READ, IFilesystem.Capability.ENTRIES);
    }

    @Override
    public WebdavFilesystem.RemoteEntry lstat(String path) {
      if (path.endsWith("/dav/")) {
        return new WebdavFilesystem.RemoteEntry(
            "dav",
            IFilesystem.EntryType.DIRECTORY,
            null,
            null,
            "fixture-root",
            "root-r1",
            IFilesystem.Capabilities.of(
                IFilesystem.Capability.READ, IFilesystem.Capability.ENTRIES),
            Map.of());
      }
      return new WebdavFilesystem.RemoteEntry(
          "README.md",
          IFilesystem.EntryType.FILE,
          5L,
          null,
          "fixture-readme",
          "readme-r1",
          IFilesystem.Capabilities.of(IFilesystem.Capability.READ),
          Map.of());
    }

    @Override
    public byte[] read(String path, long maxBytes) {
      return "hello".getBytes(java.nio.charset.StandardCharsets.UTF_8);
    }

    @Override
    public void write(
        String path,
        byte[] bytes,
        IFilesystem.WriteMode mode,
        IFilesystem.MutationContext mutation) {
      throw new UnsupportedOperationException();
    }

    @Override
    public List<WebdavFilesystem.RemoteEntry> entries(String path) {
      return List.of();
    }

    @Override
    public void mkdir(String path, IFilesystem.MutationContext mutation) {
      throw new UnsupportedOperationException();
    }

    @Override
    public void delete(String path, boolean directory, IFilesystem.MutationContext mutation) {
      throw new UnsupportedOperationException();
    }

    @Override
    public void move(
        String source,
        String target,
        boolean replace,
        boolean atomic,
        IFilesystem.MutationContext mutation) {
      throw new UnsupportedOperationException();
    }

    @Override
    public void close() {}
  }
}
