package hara.truffle;

import static org.junit.Assert.assertArrayEquals;
import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertNotNull;
import static org.junit.Assert.assertTrue;

import java.util.List;
import java.util.Map;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CompletionException;
import java.util.concurrent.CompletionStage;
import java.util.concurrent.Executors;
import java.util.concurrent.ScheduledExecutorService;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.Test;

public class KernelFilesystemRuntimeTest {
  @Test
  public void ownsFactoriesMountsAttachmentsAndFileEffects() {
    SessionKernel kernel = new SessionKernel(false, false);
    ScheduledExecutorService scheduler = Executors.newSingleThreadScheduledExecutor();
    AtomicInteger closes = new AtomicInteger();
    IFilesystemFactory.OpenContext context =
        new IFilesystemFactory.OpenContext(Runnable::run, scheduler, reference -> null);
    KernelFilesystemRuntime runtime =
        KernelFilesystemRuntime.install(kernel, context)
            .register(new FixtureFactory(closes));
    try {
      KernelFilesystemRuntime.MountId first =
          runtime.open(Map.of("kind", "fixture", "display", "first"))
              .toCompletableFuture()
              .join();
      KernelFilesystemRuntime.MountId second =
          runtime.open(Map.of("kind", "fixture", "display", "second"))
              .toCompletableFuture()
              .join();
      SessionModel.SessionId root = SessionModel.SessionId.parse("ROOT");

      runtime.attach(root, first);
      assertEquals(first, runtime.filesystem(root));
      assertEquals(1, runtime.info(first).attachments());
      assertEquals("first", runtime.info(first).descriptor().display());
      assertCloseRejected(runtime, first);

      runtime.attach(root, second);
      assertEquals(0, runtime.info(first).attachments());
      assertEquals(1, runtime.info(second).attachments());
      assertEquals(second, runtime.filesystem(root));

      IFilesystem filesystem = runtime.requireFilesystem(root);
      byte[] bytes =
          (byte[])
              FilesystemFileEffects.read(
                      filesystem, IFilesystem.CallContext.create(), "/data")
                  .toCompletableFuture()
                  .join();
      assertArrayEquals(new byte[] {0, 1, 0, 2}, bytes);
      Object listed =
          FilesystemFileEffects.list(
                  filesystem, IFilesystem.CallContext.create(), "/")
              .toCompletableFuture()
              .join();
      assertNotNull(listed);
      assertTrue(listed.toString().indexOf("/a") < listed.toString().indexOf("/z"));

      runtime.detach(root);
      runtime.closeMount(first).toCompletableFuture().join();
      runtime.closeMount(second).toCompletableFuture().join();
      assertEquals(2, closes.get());
    } finally {
      KernelFilesystemRuntime.release(kernel).toCompletableFuture().join();
      scheduler.shutdownNow();
      kernel.close();
    }
  }

  private static void assertCloseRejected(
      KernelFilesystemRuntime runtime, KernelFilesystemRuntime.MountId mountId) {
    try {
      runtime.closeMount(mountId).toCompletableFuture().join();
      throw new AssertionError("attached mount close unexpectedly succeeded");
    } catch (CompletionException expected) {
      assertTrue(expected.getCause() instanceof IllegalArgumentException);
    }
  }

  private static final class FixtureFactory implements IFilesystemFactory {
    private final AtomicInteger closes;

    FixtureFactory(AtomicInteger closes) {
      this.closes = closes;
    }

    @Override
    public String kind() {
      return "fixture";
    }

    @Override
    public CompletionStage<IFilesystem> open(
        OpenContext context, Map<String, ?> configuration) {
      String display = String.valueOf(configuration.get("display"));
      return CompletableFuture.completedFuture(new FixtureFilesystem(display, closes));
    }
  }

  private static final class FixtureFilesystem implements IFilesystem {
    private final Descriptor descriptor;
    private final AtomicInteger closes;

    FixtureFilesystem(String display, AtomicInteger closes) {
      this.descriptor =
          new Descriptor(
              "fixture",
              display,
              true,
              Capabilities.of(
                  Capability.READ,
                  Capability.ENTRIES,
                  Capability.REVISION_CHECK),
              "fixture-revision",
              Map.of());
      this.closes = closes;
    }

    @Override
    public Descriptor descriptor() {
      return descriptor;
    }

    @Override
    public CompletionStage<Entry> stat(CallContext context, String path) {
      if ("/missing".equals(path)) {
        return CompletableFuture.failedFuture(
            new FilesystemException(
                "not-found",
                "missing",
                "fixture",
                "stat",
                path,
                null,
                "missing",
                false,
                null));
      }
      if ("/".equals(path)) {
        return CompletableFuture.completedFuture(
            new Entry(
                "/",
                "",
                EntryType.DIRECTORY,
                null,
                null,
                "root",
                "tree",
                null,
                Map.of()));
      }
      return CompletableFuture.completedFuture(file(path));
    }

    @Override
    public CompletionStage<byte[]> read(CallContext context, String path) {
      return CompletableFuture.completedFuture(new byte[] {0, 1, 0, 2});
    }

    @Override
    public CompletionStage<Mutation> write(
        CallContext context,
        String path,
        byte[] bytes,
        WriteOptions options,
        MutationContext mutation) {
      return unsupported("write", path, null);
    }

    @Override
    public CompletionStage<EntryPage> entriesPage(
        CallContext context, String path, PageRequest request) {
      if (request.token() == null) {
        return CompletableFuture.completedFuture(
            new EntryPage(List.of(file("/z")), "second"));
      }
      return CompletableFuture.completedFuture(
          new EntryPage(List.of(file("/a")), null));
    }

    @Override
    public CompletionStage<Mutation> mkdir(
        CallContext context,
        String path,
        MkdirOptions options,
        MutationContext mutation) {
      return unsupported("mkdir", path, null);
    }

    @Override
    public CompletionStage<Mutation> delete(
        CallContext context,
        String path,
        DeleteOptions options,
        MutationContext mutation) {
      return unsupported("delete", path, null);
    }

    @Override
    public CompletionStage<Mutation> copy(
        CallContext context,
        String source,
        String target,
        CopyOptions options,
        MutationContext mutation) {
      return unsupported("copy", source, target);
    }

    @Override
    public CompletionStage<Mutation> move(
        CallContext context,
        String source,
        String target,
        MoveOptions options,
        MutationContext mutation) {
      return unsupported("move", source, target);
    }

    @Override
    public CompletionStage<Void> close(CallContext context) {
      closes.incrementAndGet();
      return CompletableFuture.completedFuture(null);
    }

    private static Entry file(String path) {
      return new Entry(
          path,
          path.substring(path.lastIndexOf('/') + 1),
          EntryType.FILE,
          4L,
          null,
          "blob-" + path,
          "revision-" + path,
          null,
          Map.of("provider/mode", "100644"));
    }

    private static CompletionStage<Mutation> unsupported(
        String operation, String path, String target) {
      return CompletableFuture.failedFuture(
          new FilesystemException(
              "unsupported",
              "read-only fixture",
              "fixture",
              operation,
              path,
              target,
              "read-only",
              false,
              null));
    }
  }
}
