package hara.truffle;

import hara.lang.data.Keyword;
import hara.lang.data.Map;
import hara.lang.data.Symbol;
import hara.lang.data.types.IMapType;
import hara.lang.protocol.IFn;
import hara.lang.protocol.IMetadata;
import hara.lang.protocol.IPromise;
import hara.lang.protocol.IStream;
import hara.lang.protocol.IWorkHost;
import hara.lang.protocol.IWorkRef;
import hara.lang.protocol.IWorkRun;
import java.util.ArrayList;
import java.util.List;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CompletionException;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.ConcurrentMap;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicReference;

/** Process-owned native work host and live run registry. */
public final class HaraWorkHost implements IWorkHost {
  static final HaraWorkHost INSTANCE = new HaraWorkHost();

  private static final Keyword ID = Keyword.create("id");
  private static final Keyword WORK_ID = Keyword.create("work", "id");
  private static final Keyword STATUS_WORK_ID = Keyword.create("work-id");
  private static final Keyword RUN_ID = Keyword.create("run", "id");
  private static final Keyword EXECUTE = Keyword.create("work", "execute");
  private static final Keyword TYPE = Keyword.create("type");
  private static final Keyword SCOPE = Keyword.create("scope");
  private static final Keyword STATE = Keyword.create("state");
  private static final Keyword RUN_COUNT = Keyword.create("run-count");
  private static final Keyword WORK_HOST = Keyword.create("work-host");
  private static final Keyword PROCESS = Keyword.create("process");
  private static final Keyword STARTED = Keyword.create("started");
  private static final Keyword STOPPED = Keyword.create("stopped");
  private static final Keyword CREATED = Keyword.create("created");
  private static final Keyword QUEUED = Keyword.create("queued");
  private static final Keyword RUNNING = Keyword.create("running");
  private static final Keyword COMPLETED = Keyword.create("completed");
  private static final Keyword FAILED = Keyword.create("failed");
  private static final Keyword CANCELLED = Keyword.create("cancelled");
  private static final Keyword STARTED_AT = Keyword.create("started-at");
  private static final Keyword FINISHED_AT = Keyword.create("finished-at");
  private static final Keyword ERROR = Keyword.create("error");

  private final ConcurrentMap<Object, HaraWorkRun> runs = new ConcurrentHashMap<>();
  private final AtomicBoolean started = new AtomicBoolean(true);

  private HaraWorkHost() {}

  static HaraWorkHost instance() {
    return INSTANCE;
  }

  @Override
  public IWorkRun workSubmit(Object work, Object input, Object options) {
    return workSubmit(HaraLanguage.currentContext(), work, input, options);
  }

  IWorkRun workSubmit(HaraContext context, Object work, Object input, Object options) {
    if (!started.get()) {
      throw new HaraException("Native work host is stopped");
    }
    Object executor = option(options, EXECUTE);
    if (executor == null && work instanceof IFn<?, ?, ?>) {
      executor = work;
    }
    if (executor == null) {
      throw new HaraException(
          "work-submit requires callable work or a :work/execute adapter");
    }

    Object requestedId =
        firstNonNull(
            option(options, ID),
            firstNonNull(option(options, RUN_ID), option(options, WORK_ID)));
    Object runId = requestedId == null ? nextRunId() : validateRunId(requestedId);
    HaraWorkRun run = new HaraWorkRun(context, runId, work, input, options, executor);
    HaraWorkRun previous = runs.putIfAbsent(runId, run);
    if (previous != null) {
      throw new HaraException("Work run ID is already active: " + runId);
    }
    run.start();
    return run;
  }

  @Override
  public IWorkRun workResolve(Object reference) {
    Object runId = referenceId(reference);
    HaraWorkRun run = runs.get(runId);
    if (run == null) {
      throw new HaraException("Unknown work run: " + runId);
    }
    return run;
  }

  @Override
  public IMetadata getProps() {
    return Map.Standard.from(null, TYPE, WORK_HOST, SCOPE, PROCESS);
  }

  @Override
  public IMetadata getStatus() {
    return Map.Standard.from(
        null, STATE, started.get() ? STARTED : STOPPED, RUN_COUNT, (long) runs.size());
  }

  @Override
  public boolean isStarted() {
    return started.get();
  }

  @Override
  public boolean isStopped() {
    return !started.get();
  }

  @Override
  public IWorkHost start() {
    started.set(true);
    return this;
  }

  @Override
  public IWorkHost stop() {
    if (started.compareAndSet(true, false)) {
      for (HaraWorkRun run : runs.values()) {
        run.cancel(Keyword.create("host-stopped"));
      }
    }
    return this;
  }

  private Object nextRunId() {
    Object candidate;
    do {
      candidate = UUID.randomUUID().toString();
    } while (runs.containsKey(candidate));
    return candidate;
  }

  private static Object validateRunId(Object value) {
    if (value == null) {
      throw new HaraException("Work run ID cannot be nil");
    }
    if (value instanceof String string) {
      if (string.isBlank()) {
        throw new HaraException("Work run ID cannot be blank");
      }
      return string;
    }
    if (value instanceof Keyword || value instanceof Symbol || value instanceof Number) {
      return value;
    }
    throw new HaraException(
        "Work run ID must be a string, keyword, symbol, or number: "
            + value.getClass().getName());
  }

  private static Object referenceId(Object reference) {
    if (reference instanceof IWorkRef workRef) {
      return validateRunId(workRef.workId());
    }
    Object workId = option(reference, WORK_ID);
    if (workId != null) return validateRunId(workId);
    Object runId = option(reference, RUN_ID);
    if (runId != null) return validateRunId(runId);
    Object id = option(reference, ID);
    return validateRunId(id == null ? reference : id);
  }

  @SuppressWarnings({"rawtypes", "unchecked"})
  private static Object option(Object options, Keyword key) {
    if (options instanceof IMapType map) {
      return map.lookup(key);
    }
    if (options instanceof java.util.Map map) {
      return map.get(key);
    }
    return null;
  }

  private static Object firstNonNull(Object first, Object second) {
    return first == null ? second : first;
  }

  private static final class HaraWorkRun implements IWorkRun {
    private final HaraContext context;
    private final Object runId;
    private final Object work;
    private final Object input;
    private final Object options;
    private final Object executor;
    private final CompletableFuture<Object> resultFuture = new CompletableFuture<>();
    private final IPromise result;
    private final AtomicReference<RunSnapshot> snapshot;
    private final AtomicReference<Thread> worker = new AtomicReference<>();

    HaraWorkRun(
        HaraContext context,
        Object runId,
        Object work,
        Object input,
        Object options,
        Object executor) {
      this.context = context;
      this.runId = runId;
      this.work = work;
      this.input = input;
      this.options = options;
      this.executor = executor;
      this.result = (IPromise) context.promiseValue(resultFuture);
      this.snapshot = new AtomicReference<>(new RunSnapshot(CREATED, 0L, 0L, null));
    }

    void start() {
      long startedAt = System.currentTimeMillis();
      RunSnapshot current = snapshot.get();
      if (!CREATED.equals(current.state)
          || !snapshot.compareAndSet(current, new RunSnapshot(QUEUED, startedAt, 0L, null))) {
        return;
      }
      Thread thread = Thread.ofVirtual().name("hara-work-" + runId).unstarted(this::execute);
      worker.set(thread);
      thread.start();
    }

    private void execute() {
      if (!transition(QUEUED, RUNNING, null)) {
        return;
      }
      try {
        Object value =
            context.invokeInContext(
                () -> context.invokeCallable(executor, new Object[] {work, input, options, runId}));
        if (value instanceof IPromise promise) {
          value = promise.deref();
        }
        complete(value);
      } catch (Throwable error) {
        fail(unwrap(error));
      }
    }

    private void complete(Object value) {
      if (transitionTerminal(COMPLETED, null)) {
        resultFuture.complete(value);
      }
    }

    private void fail(Throwable error) {
      if (transitionTerminal(FAILED, error)) {
        resultFuture.completeExceptionally(error);
      }
    }

    boolean cancel(Object reason) {
      HaraException failure = new HaraException("Work run cancelled: " + String.valueOf(reason));
      if (!transitionTerminal(CANCELLED, failure)) {
        return false;
      }
      Thread thread = worker.get();
      if (thread != null) thread.interrupt();
      resultFuture.completeExceptionally(failure);
      return true;
    }

    private boolean transition(Keyword expected, Keyword next, Throwable error) {
      while (true) {
        RunSnapshot current = snapshot.get();
        if (!expected.equals(current.state) || current.terminal()) return false;
        RunSnapshot update = new RunSnapshot(next, current.startedAt, 0L, error);
        if (snapshot.compareAndSet(current, update)) return true;
      }
    }

    private boolean transitionTerminal(Keyword state, Throwable error) {
      while (true) {
        RunSnapshot current = snapshot.get();
        if (current.terminal()) return false;
        RunSnapshot update =
            new RunSnapshot(state, current.startedAt, System.currentTimeMillis(), error);
        if (snapshot.compareAndSet(current, update)) return true;
      }
    }

    @Override
    public Object workId() {
      return runId;
    }

    @Override
    public Object workStatus() {
      RunSnapshot current = snapshot.get();
      List<Object> entries = new ArrayList<>();
      entries.add(STATE);
      entries.add(current.state);
      entries.add(STATUS_WORK_ID);
      entries.add(runId);
      if (current.startedAt != 0L) {
        entries.add(STARTED_AT);
        entries.add(current.startedAt);
      }
      if (current.finishedAt != 0L) {
        entries.add(FINISHED_AT);
        entries.add(current.finishedAt);
      }
      if (current.error != null) {
        entries.add(ERROR);
        entries.add(current.error);
      }
      return Map.Standard.from(null, entries.toArray());
    }

    @Override
    public IPromise workResult() {
      return result;
    }

    @Override
    public IStream workEvents(Object options) {
      return new EmptyWorkStream(context);
    }

    @Override
    public IPromise workCancel(Object reason) {
      return (IPromise)
          context.promiseValue(CompletableFuture.completedFuture(cancel(reason)));
    }

    @Override
    public boolean closed() {
      return snapshot.get().terminal();
    }

    private static Throwable unwrap(Throwable error) {
      Throwable current = error;
      while ((current instanceof CompletionException) && current.getCause() != null) {
        current = current.getCause();
      }
      return current;
    }
  }

  private static final class EmptyWorkStream implements IStream {
    private final IPromise end;

    EmptyWorkStream(HaraContext context) {
      this.end = (IPromise) context.promiseValue(CompletableFuture.completedFuture(null));
    }

    @Override
    public Object next() {
      return end;
    }

    @Override
    public void close() {}
  }

  private record RunSnapshot(Keyword state, long startedAt, long finishedAt, Throwable error) {
    boolean terminal() {
      return COMPLETED.equals(state) || FAILED.equals(state) || CANCELLED.equals(state);
    }
  }
}
