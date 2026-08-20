package hara.truffle;

import static org.junit.Assert.assertArrayEquals;
import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertSame;
import static org.junit.Assert.assertThrows;
import static org.junit.Assert.assertTrue;

import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CompletionException;
import java.util.concurrent.CompletionStage;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.Test;

public class FilesystemRuntimeBindingTest {
  @Test
  public void dispatchUsesTheExactProviderAndProducesTraceableCalls() {
    ControlledFilesystem filesystem = ControlledFilesystem.readWrite();
    FilesystemRuntimeBinding binding = new FilesystemRuntimeBinding(filesystem);

    FilesystemRuntimeBinding.Pending<byte[]> pending = binding.read("/value.bin");
    assertEquals(1, filesystem.readCalls.get());
    assertEquals("/value.bin", filesystem.lastPath);
    assertTrue(filesystem.lastContext.traceId().startsWith("filesystem/fixture/read/"));
    assertEquals(1, binding.pendingCount());

    filesystem.readStage.complete(new byte[] {0, 1, 0, (byte) 255});
    assertArrayEquals(new byte[] {0, 1, 0, (byte) 255}, join(pending.future()));
    assertEquals(0, binding.pendingCount());
    assertSame(filesystem, binding.filesystem());
    assertEquals("fixture", binding.descriptor().kind());
  }

  @Test
  public void cancellationSettlesOnceAndIgnoresLateProviderSuccess() {
    ControlledFilesystem filesystem = ControlledFilesystem.readWrite();
    FilesystemRuntimeBinding binding = new FilesystemRuntimeBinding(filesystem);
    FilesystemRuntimeBinding.Pending<byte[]> pending = binding.read("/slow.bin");
    IFilesystem.CallContext call = filesystem.lastContext;

    assertTrue(pending.cancel());
    FilesystemException cancelled =
        assertThrows(FilesystemException.class, () -> join(pending.future()));
    assertEquals("cancelled", cancelled.code());
    assertTrue(call.cancelled());
    assertFalse(pending.cancel());

    filesystem.readStage.complete(new byte[] {7});
    FilesystemException stillCancelled =
        assertThrows(FilesystemException.class, () -> join(pending.future()));
    assertEquals("cancelled", stillCancelled.code());
    assertEquals(0, binding.pendingCount());
  }

  @Test
  public void detachClosesOnlyTheBindingAndRejectsLateSettlementsAndReuse() {
    ControlledFilesystem filesystem = ControlledFilesystem.readWrite();
    FilesystemRuntimeBinding binding = new FilesystemRuntimeBinding(filesystem);
    FilesystemRuntimeBinding.Pending<byte[]> pending = binding.read("/pending.bin");

    binding.close();
    binding.close();
    assertTrue(binding.closed());
    assertEquals(0, binding.pendingCount());
    FilesystemException closed =
        assertThrows(FilesystemException.class, () -> join(pending.future()));
    assertEquals("provider-closed", closed.code());
    assertTrue(filesystem.lastContext.cancelled());

    filesystem.readStage.complete(new byte[] {1});
    FilesystemException late =
        assertThrows(FilesystemException.class, () -> join(pending.future()));
    assertEquals("provider-closed", late.code());

    FilesystemException reused =
        assertThrows(
            FilesystemException.class,
            () -> join(binding.read("/after-close").future()));
    assertEquals("provider-closed", reused.code());
    assertEquals(1, filesystem.readCalls.get());
    assertEquals(0, filesystem.closeCalls.get());
  }

  @Test
  public void missingCapabilitiesRejectBeforeProviderInvocation() {
    ControlledFilesystem readOnly =
        new ControlledFilesystem(Set.of(IFilesystem.Capability.READ));
    FilesystemRuntimeBinding binding = new FilesystemRuntimeBinding(readOnly);

    assertUnsupported(
        binding
            .write(
                "/new",
                new byte[] {1},
                new IFilesystem.WriteOptions(IFilesystem.WriteMode.CREATE, false),
                IFilesystem.MutationContext.none())
            .future(),
        "capability-unavailable:write");
    assertEquals(0, readOnly.writeCalls.get());

    ControlledFilesystem noAppend =
        new ControlledFilesystem(
            Set.of(IFilesystem.Capability.READ, IFilesystem.Capability.WRITE));
    assertUnsupported(
        new FilesystemRuntimeBinding(noAppend)
            .write(
                "/append",
                new byte[] {1},
                new IFilesystem.WriteOptions(IFilesystem.WriteMode.APPEND, false),
                IFilesystem.MutationContext.none())
            .future(),
        "capability-unavailable:append");
    assertEquals(0, noAppend.writeCalls.get());

    ControlledFilesystem copyOnly =
        new ControlledFilesystem(Set.of(IFilesystem.Capability.COPY));
    assertUnsupported(
        new FilesystemRuntimeBinding(copyOnly)
            .copy(
                "/source",
                "/target",
                new IFilesystem.CopyOptions(false, false, true),
                IFilesystem.MutationContext.none())
            .future(),
        "capability-unavailable:preserve-modified");
    assertEquals(0, copyOnly.copyCalls.get());

    ControlledFilesystem moveOnly =
        new ControlledFilesystem(Set.of(IFilesystem.Capability.MOVE));
    assertUnsupported(
        new FilesystemRuntimeBinding(moveOnly)
            .move(
                "/source",
                "/target",
                new IFilesystem.MoveOptions(false, false, true),
                IFilesystem.MutationContext.none())
            .future(),
        "capability-unavailable:atomic-move");
    assertEquals(0, moveOnly.moveCalls.get());
  }

  @Test
  public void unknownProviderFailuresAreNormalizedWithoutMessageLeakage() {
    ControlledFilesystem filesystem = ControlledFilesystem.readWrite();
    FilesystemRuntimeBinding binding = new FilesystemRuntimeBinding(filesystem);
    FilesystemRuntimeBinding.Pending<byte[]> pending = binding.read("/secret");
    filesystem.readStage.completeExceptionally(
        new IllegalStateException("credential=secret-token"));

    FilesystemException failure =
        assertThrows(FilesystemException.class, () -> join(pending.future()));
    assertEquals("io", failure.code());
    assertEquals("filesystem operation failed", failure.getMessage());
    assertEquals("IllegalStateException", failure.providerCode());
    assertFalse(failure.data().toString().contains("secret-token"));
  }

  private static final class ControlledFilesystem implements IFilesystem {
    private final Descriptor descriptor;
    final CompletableFuture<byte[]> readStage = new CompletableFuture<>();
    final AtomicInteger readCalls = new AtomicInteger();
    final AtomicInteger writeCalls = new AtomicInteger();
    final AtomicInteger copyCalls = new AtomicInteger();
    final AtomicInteger moveCalls = new AtomicInteger();
    final AtomicInteger closeCalls = new AtomicInteger();
    volatile CallContext lastContext;
    volatile String lastPath;

    ControlledFilesystem(Set<Capability> capabilities) {
      descriptor =
          new Descriptor(
              "fixture",
              "fixture",
              false,
              new Capabilities(capabilities),
              "1",
              Map.of());
    }

    static ControlledFilesystem readWrite() {
      return new ControlledFilesystem(
          Set.of(
              Capability.READ,
              Capability.WRITE,
              Capability.APPEND,
              Capability.ENTRIES,
              Capability.MKDIR,
              Capability.DELETE,
              Capability.COPY,
              Capability.MOVE));
    }

    @Override
    public Descriptor descriptor() {
      return descriptor;
    }

    @Override
    public CompletionStage<Entry> stat(CallContext context, String path) {
      lastContext = context;
      lastPath = path;
      return CompletableFuture.completedFuture(
          new Entry(
              path,
              HaraLogicalPath.fileName(path),
              EntryType.FILE,
              0L,
              null,
              path,
              "1",
              null,
              Map.of()));
    }

    @Override
    public CompletionStage<byte[]> read(CallContext context, String path) {
      readCalls.incrementAndGet();
      lastContext = context;
      lastPath = path;
      return readStage;
    }

    @Override
    public CompletionStage<Mutation> write(
        CallContext context,
        String path,
        byte[] bytes,
        WriteOptions options,
        MutationContext mutation) {
      writeCalls.incrementAndGet();
      return CompletableFuture.completedFuture(Mutation.path(path));
    }

    @Override
    public CompletionStage<EntryPage> entriesPage(
        CallContext context, String path, PageRequest request) {
      return CompletableFuture.completedFuture(new EntryPage(List.of(), null));
    }

    @Override
    public CompletionStage<Mutation> mkdir(
        CallContext context,
        String path,
        MkdirOptions options,
        MutationContext mutation) {
      return CompletableFuture.completedFuture(Mutation.path(path));
    }

    @Override
    public CompletionStage<Mutation> delete(
        CallContext context,
        String path,
        DeleteOptions options,
        MutationContext mutation) {
      return CompletableFuture.completedFuture(Mutation.path(path));
    }

    @Override
    public CompletionStage<Mutation> copy(
        CallContext context,
        String source,
        String target,
        CopyOptions options,
        MutationContext mutation) {
      copyCalls.incrementAndGet();
      return CompletableFuture.completedFuture(Mutation.path(target));
    }

    @Override
    public CompletionStage<Mutation> move(
        CallContext context,
        String source,
        String target,
        MoveOptions options,
        MutationContext mutation) {
      moveCalls.incrementAndGet();
      return CompletableFuture.completedFuture(Mutation.path(target));
    }

    @Override
    public CompletionStage<Void> close(CallContext context) {
      closeCalls.incrementAndGet();
      return CompletableFuture.completedFuture(null);
    }
  }

  private static void assertUnsupported(
      CompletionStage<?> stage, String providerCode) {
    FilesystemException failure =
        assertThrows(FilesystemException.class, () -> join(stage));
    assertEquals("unsupported", failure.code());
    assertEquals(providerCode, failure.providerCode());
  }

  private static <T> T join(CompletionStage<T> stage) {
    try {
      return stage.toCompletableFuture().join();
    } catch (CompletionException error) {
      Throwable cause = unwrap(error);
      if (cause instanceof RuntimeException runtime) throw runtime;
      throw error;
    }
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
}
