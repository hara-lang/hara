package hara.truffle;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertSame;
import static org.junit.Assert.assertTrue;

import java.util.Map;
import java.util.concurrent.CompletionException;
import org.junit.Test;

public class WebDavFilesystemSessionKernelTest {
  @Test
  public void davMountFlowsThroughPublicFileDispatch() throws Exception {
    try (WebDavFilesystemFixture fixture = new WebDavFilesystemFixture();
        SessionKernel kernel = kernel(fixture)) {
      kernel.registerFilesystemProvider(new WebDavFilesystem.Factory());
      SessionModel.SessionMountId mount =
          join(
              kernel.createFilesystem(
                  "webdav",
                  Map.of(
                      "credential-ref", "dav:test",
                      "origin", WebDavFilesystemFixture.ORIGIN,
                      "root", WebDavFilesystemFixture.ROOT,
                      "display", "DAV fixture")));
      SessionKernel.Session session = kernel.create(SessionModel.SessionId.parse("DAV-READ"));

      kernel.attachFilesystem(session.id(), mount);
      FilesystemRuntimeBinding binding = kernel.filesystemRuntime(session.id());
      assertTrue(binding.filesystem() instanceof WebDavFilesystem);
      assertSame(binding.filesystem(), kernel.filesystemRuntime(session.id()).filesystem());
      assertEquals(
          "hello",
          session
              .eval(
                  "(std.foundation.string/decode-utf8"
                      + " (deref (File/read \"/README.md\")))")
              .asString());
      assertEquals(
          "directory",
          session.eval("(name (:type (deref (File/stat \"/docs\"))))").asString());

      SessionKernel.FilesystemInfo info = kernel.filesystemInfo(mount);
      assertEquals("webdav", info.kind());
      assertEquals("DAV fixture", info.display());
      assertFalse(info.readOnly());
      assertFalse(info.sourceLoadable());
      assertEquals(1, info.attachments());
      assertTrue(info.capabilities().contains(IFilesystem.Capability.READ));
      assertTrue(info.extensions().containsKey("provider/hierarchical?"));
      assertFalse(info.toString().contains("dav:test"));
      assertFalse(info.toString().contains(WebDavFilesystemFixture.ORIGIN));
      assertFalse(info.toString().contains(WebDavFilesystemFixture.ROOT));

      kernel.detachFilesystem(session.id());
      assertTrue(binding.closed());
      assertEquals(0, kernel.filesystemInfo(mount).attachments());
      kernel.closeFilesystem(mount);
    }
  }

  private static SessionKernel kernel(WebDavFilesystemFixture fixture) {
    return new SessionKernel(
        true,
        false,
        false,
        null,
        reference -> {
          assertEquals("dav:test", reference);
          return fixture.client;
        });
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
}
