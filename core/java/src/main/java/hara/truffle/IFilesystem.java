package hara.truffle;

import java.time.Duration;
import java.time.Instant;
import java.util.ArrayList;
import java.util.Collections;
import java.util.EnumSet;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Set;
import java.util.concurrent.CompletionStage;
import java.util.concurrent.CopyOnWriteArrayList;
import java.util.concurrent.atomic.AtomicBoolean;

/**
 * An opened, provider-neutral filesystem mounted at a Session's logical root.
 *
 * <p>This is a trusted runtime interface. It is not exposed as a Hara protocol and it carries no
 * provider-construction or credential authority.
 */
public interface IFilesystem {
  enum Capability {
    READ,
    WRITE,
    ENTRIES,
    MKDIR,
    DELETE,
    COPY,
    MOVE,
    APPEND,
    ATOMIC_MOVE,
    PRESERVE_MODIFIED,
    REVISION_CHECK,
    TRANSACTIONS,
    WATCH,
    RANDOM_ACCESS
  }

  record Capabilities(Set<Capability> values) {
    public Capabilities {
      values = values == null || values.isEmpty() ? Set.of() : Collections.unmodifiableSet(EnumSet.copyOf(values));
    }

    static Capabilities of(Capability... values) {
      return new Capabilities(values.length == 0 ? Set.of() : EnumSet.of(values[0], values));
    }

    boolean contains(Capability capability) {
      return values.contains(capability);
    }
  }

  record Descriptor(
      String kind,
      String display,
      boolean readOnly,
      Capabilities capabilities,
      String revision,
      Map<String, Object> extensions,
      boolean sourceLoadable) {
    public Descriptor {
      kind = Objects.requireNonNull(kind, "filesystem kind");
      display = Objects.requireNonNull(display, "filesystem display");
      capabilities = Objects.requireNonNull(capabilities, "filesystem capabilities");
      extensions = extensions == null || extensions.isEmpty() ? Map.of() : Map.copyOf(extensions);
    }
  }

  record Entry(
      String path,
      String name,
      String type,
      Long size,
      Long modifiedAt,
      Map<String, Object> extensions) {
    public Entry {
      path = Objects.requireNonNull(path, "entry path");
      name = Objects.requireNonNull(name, "entry name");
      type = Objects.requireNonNull(type, "entry type");
      extensions = extensions == null || extensions.isEmpty() ? Map.of() : Map.copyOf(extensions);
    }
  }

  record PageRequest(String cursor, int limit) {
    public PageRequest {
      if (limit <= 0) throw new IllegalArgumentException("filesystem page limit must be positive");
    }
  }

  record EntryPage(List<Entry> entries, String nextCursor) {
    public EntryPage {
      entries = List.copyOf(Objects.requireNonNull(entries, "filesystem entries"));
    }
  }

  record Mutation(String revision, Map<String, Object> extensions) {
    public Mutation {
      extensions = extensions == null || extensions.isEmpty() ? Map.of() : Map.copyOf(extensions);
    }
  }

  final class CallContext {
    private final AtomicBoolean cancelled = new AtomicBoolean();
    private final Instant deadline;
    private final CopyOnWriteArrayList<Runnable> cancellationHooks = new CopyOnWriteArrayList<>();

    public CallContext() {
      this(null);
    }

    public CallContext(Instant deadline) {
      this.deadline = deadline;
    }

    public static CallContext withTimeout(Duration timeout) {
      Objects.requireNonNull(timeout, "filesystem timeout");
      return new CallContext(Instant.now().plus(timeout));
    }

    public boolean cancelled() {
      return cancelled.get();
    }

    public Instant deadline() {
      return deadline;
    }

    public void onCancel(Runnable hook) {
      Objects.requireNonNull(hook, "cancellation hook");
      if (cancelled()) hook.run();
      else {
        cancellationHooks.add(hook);
        if (cancelled() && cancellationHooks.remove(hook)) hook.run();
      }
    }

    public void cancel() {
      if (!cancelled.compareAndSet(false, true)) return;
      ArrayList<Runnable> hooks = new ArrayList<>(cancellationHooks);
      cancellationHooks.clear();
      for (Runnable hook : hooks) hook.run();
    }

    public void check() {
      if (cancelled()) throw new IllegalStateException("FILESYSTEM_OPERATION_CANCELLED");
      if (deadline != null && !Instant.now().isBefore(deadline))
        throw new IllegalStateException("FILESYSTEM_OPERATION_TIMED_OUT");
    }
  }

  record MutationContext(CallContext call, String expectedRevision) {
    public MutationContext {
      call = Objects.requireNonNull(call, "filesystem call context");
    }
  }

  Descriptor descriptor();

  CompletionStage<byte[]> read(CallContext context, String path);

  CompletionStage<Mutation> write(
      MutationContext context, String path, byte[] data, boolean append, boolean createParents);

  CompletionStage<EntryPage> entries(CallContext context, String path, PageRequest page);

  CompletionStage<Mutation> mkdir(MutationContext context, String path, boolean recursive);

  CompletionStage<Mutation> delete(MutationContext context, String path, boolean recursive);

  CompletionStage<Mutation> copy(
      MutationContext context, String source, String target, boolean replaceExisting);

  CompletionStage<Mutation> move(
      MutationContext context, String source, String target, boolean replaceExisting);

  CompletionStage<Void> close(CallContext context);
}
