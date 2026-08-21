package hara.truffle;

import static org.junit.Assert.assertArrayEquals;
import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;
import static org.junit.Assert.fail;

import java.net.URI;
import java.nio.charset.StandardCharsets;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.concurrent.CompletionException;
import java.util.concurrent.Executors;
import java.util.concurrent.ScheduledExecutorService;
import org.junit.Test;

public class WebDavFilesystemTest {
  @Test
  public void authenticatedHttpsMountRedactsAuthorityAndAdvertisesOnlyProvenCapabilities()
      throws Exception {
    try (WebDavFilesystemFixture fixture = new WebDavFilesystemFixture();
        FixtureExecutors executors = new FixtureExecutors()) {
      fixture.client.authenticated = false;
      try {
        join(
            new WebDavFilesystem.Factory()
                .open(executors.context(reference -> fixture.client), config("secret:dav")));
        fail("expected authentication failure");
      } catch (FilesystemException error) {
        assertEquals("authentication-failed", error.code());
        assertFalse(error.data().toString().contains("secret:dav"));
      }

      fixture.client.authenticated = true;
      IFilesystem filesystem =
          join(
              new WebDavFilesystem.Factory()
                  .open(executors.context(reference -> fixture.client), config("secret:dav")));
      assertEquals("webdav", filesystem.descriptor().kind());
      assertEquals("DAV fixture", filesystem.descriptor().display());
      assertTrue(filesystem.capabilities().contains(IFilesystem.Capability.READ));
      assertTrue(filesystem.capabilities().contains(IFilesystem.Capability.MKDIR));
      assertTrue(filesystem.capabilities().contains(IFilesystem.Capability.MOVE));
      assertFalse(filesystem.capabilities().contains(IFilesystem.Capability.APPEND));
      assertFalse(filesystem.capabilities().contains(IFilesystem.Capability.ATOMIC_MOVE));
      assertFalse(filesystem.capabilities().contains(IFilesystem.Capability.TRANSACTIONS));
      assertFalse(filesystem.descriptor().toString().contains("secret:dav"));
      assertFalse(filesystem.descriptor().toString().contains(WebDavFilesystemFixture.ORIGIN));
      assertFalse(filesystem.descriptor().toString().contains(WebDavFilesystemFixture.ROOT));
      join(filesystem.close(IFilesystem.CallContext.create()));
    }
  }

  @Test
  public void collectionsExactBytesPaginationAndMutationsRemainHierarchical() throws Exception {
    try (WebDavFilesystemFixture fixture = new WebDavFilesystemFixture();
        FixtureExecutors executors = new FixtureExecutors()) {
      IFilesystem filesystem = open(fixture, executors);

      assertEquals(
          IFilesystem.EntryType.DIRECTORY,
          join(filesystem.stat(IFilesystem.CallContext.create(), "/docs")).type());
      assertArrayEquals(
          "hello".getBytes(StandardCharsets.UTF_8),
          join(filesystem.read(IFilesystem.CallContext.create(), "/README.md")));

      IFilesystem.EntryPage first =
          join(
              filesystem.entriesPage(
                  IFilesystem.CallContext.create(),
                  "/docs",
                  new IFilesystem.PageRequest(null, 1)));
      assertEquals(List.of("/docs/a.bin"), first.entries().stream().map(IFilesystem.Entry::path).toList());
      IFilesystem.EntryPage second =
          join(
              filesystem.entriesPage(
                  IFilesystem.CallContext.create(),
                  "/docs",
                  new IFilesystem.PageRequest(first.nextToken(), 1)));
      assertEquals(List.of("/docs/b.bin"), second.entries().stream().map(IFilesystem.Entry::path).toList());

      IFilesystem.Mutation created =
          join(
              filesystem.write(
                  IFilesystem.CallContext.create(),
                  "/new.bin",
                  new byte[] {0, 1, (byte) 255},
                  new IFilesystem.WriteOptions(IFilesystem.WriteMode.CREATE, false),
                  IFilesystem.MutationContext.none()));
      assertEquals("/new.bin", created.path());
      assertArrayEquals(
          new byte[] {0, 1, (byte) 255},
          fixture.client.bytes(URI.create(WebDavFilesystemFixture.ORIGIN + "/files/new.bin")));

      join(
          filesystem.mkdir(
              IFilesystem.CallContext.create(),
              "/empty",
              new IFilesystem.MkdirOptions(false, false),
              IFilesystem.MutationContext.none()));
      assertTrue(
          fixture.client.exists(URI.create(WebDavFilesystemFixture.ORIGIN + "/files/empty")));

      join(
          filesystem.copy(
              IFilesystem.CallContext.create(),
              "/new.bin",
              "/copy.bin",
              new IFilesystem.CopyOptions(false, false, false),
              IFilesystem.MutationContext.none()));
      assertTrue(
          fixture.client.exists(URI.create(WebDavFilesystemFixture.ORIGIN + "/files/copy.bin")));

      join(
          filesystem.move(
              IFilesystem.CallContext.create(),
              "/copy.bin",
              "/moved.bin",
              new IFilesystem.MoveOptions(false, false, false),
              IFilesystem.MutationContext.none()));
      assertFalse(
          fixture.client.exists(URI.create(WebDavFilesystemFixture.ORIGIN + "/files/copy.bin")));
      assertTrue(
          fixture.client.exists(URI.create(WebDavFilesystemFixture.ORIGIN + "/files/moved.bin")));

      try {
        join(
            filesystem.move(
                IFilesystem.CallContext.create(),
                "/moved.bin",
                "/atomic.bin",
                new IFilesystem.MoveOptions(false, false, true),
                IFilesystem.MutationContext.none()));
        fail("expected atomic move rejection");
      } catch (FilesystemException error) {
        assertEquals("unsupported", error.code());
        assertEquals("atomic-move-unavailable", error.providerCode());
      }
      join(filesystem.close(IFilesystem.CallContext.create()));
    }
  }

  @Test
  public void etagConflictsAndReturnedHrefEscapesFailClosed() throws Exception {
    try (WebDavFilesystemFixture fixture = new WebDavFilesystemFixture();
        FixtureExecutors executors = new FixtureExecutors()) {
      IFilesystem filesystem = open(fixture, executors);
      IFilesystem.Entry readme =
          join(filesystem.stat(IFilesystem.CallContext.create(), "/README.md"));
      join(
          filesystem.write(
              IFilesystem.CallContext.create(),
              "/README.md",
              "updated".getBytes(StandardCharsets.UTF_8),
              new IFilesystem.WriteOptions(IFilesystem.WriteMode.REPLACE, false),
              new IFilesystem.MutationContext(readme.revision(), null)));
      try {
        join(
            filesystem.write(
                IFilesystem.CallContext.create(),
                "/README.md",
                "stale".getBytes(StandardCharsets.UTF_8),
                new IFilesystem.WriteOptions(IFilesystem.WriteMode.REPLACE, false),
                new IFilesystem.MutationContext(readme.revision(), null)));
        fail("expected revision conflict");
      } catch (FilesystemException error) {
        assertEquals("conflict", error.code());
      }

      fixture.client.overridePage =
          new WebDavFilesystem.ResourcePage(
              List.of(
                  new WebDavFilesystem.Resource(
                      "https://evil.example.test/stolen",
                      "stolen",
                      IFilesystem.EntryType.FILE,
                      1L,
                      1L,
                      "evil",
                      new IFilesystem.Capabilities(Set.of(IFilesystem.Capability.READ)),
                      Map.of())),
              null);
      try {
        join(
            filesystem.entriesPage(
                IFilesystem.CallContext.create(), "/docs", IFilesystem.PageRequest.first()));
        fail("expected returned href confinement failure");
      } catch (FilesystemException error) {
        assertEquals("outside-root", error.code());
      }

      try {
        WebDavFilesystem.mountedRoot(
            WebDavFilesystemFixture.ORIGIN, "/files/%2e%2e/private");
        fail("expected encoded traversal rejection");
      } catch (IllegalArgumentException expected) {
        assertTrue(expected.getMessage().contains("escapes"));
      }
      join(filesystem.close(IFilesystem.CallContext.create()));
    }
  }

  private static IFilesystem open(WebDavFilesystemFixture fixture, FixtureExecutors executors) {
    return join(
        new WebDavFilesystem.Factory()
            .open(executors.context(reference -> fixture.client), config("dav:test")));
  }

  private static Map<String, Object> config(String credentialReference) {
    return Map.of(
        "credential-ref", credentialReference,
        "origin", WebDavFilesystemFixture.ORIGIN,
        "root", WebDavFilesystemFixture.ROOT,
        "display", "DAV fixture",
        "operation-timeout-ms", 5_000,
        "max-transfer-bytes", 1024 * 1024);
  }

  private static <T> T join(java.util.concurrent.CompletionStage<T> stage) {
    try {
      return stage.toCompletableFuture().join();
    } catch (CompletionException error) {
      Throwable cause = error.getCause();
      if (cause instanceof RuntimeException runtime) throw runtime;
      throw error;
    }
  }

  private static final class FixtureExecutors implements AutoCloseable {
    private final java.util.concurrent.ExecutorService io = Executors.newCachedThreadPool();
    private final ScheduledExecutorService scheduler = Executors.newSingleThreadScheduledExecutor();

    IFilesystemFactory.OpenContext context(IFilesystemFactory.CredentialResolver credentials) {
      return new IFilesystemFactory.OpenContext(io, scheduler, credentials);
    }

    @Override
    public void close() {
      io.shutdownNow();
      scheduler.shutdownNow();
    }
  }
}
