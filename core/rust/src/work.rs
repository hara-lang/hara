use crate::core::{Promise, PromiseRejection, Value};
use crate::lang::protocol::IComponent;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::rc::{Rc, Weak};
use std::time::{SystemTime, UNIX_EPOCH};

/// Validated portable identifier for one native work run.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct WorkId(String);

impl WorkId {
    pub fn new(value: impl Into<String>) -> Result<Self, String> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err("work run ID cannot be blank".into());
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for WorkId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Monotonic state of a live native work run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkRunState {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl WorkRunState {
    pub fn terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }
}

/// Non-blocking status snapshot for a live work run.
#[derive(Clone, Debug, PartialEq)]
pub struct WorkRunStatus {
    pub id: WorkId,
    pub state: WorkRunState,
    pub started_at_millis: u64,
    pub finished_at_millis: Option<u64>,
    pub error: Option<PromiseRejection>,
}

/// Process-host lifecycle metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkHostStatus {
    pub state: &'static str,
    pub run_count: usize,
    pub queued_count: usize,
}

type WorkTask = Box<dyn FnOnce() -> Result<Value, PromiseRejection>>;

struct PendingWork {
    id: WorkId,
    task: WorkTask,
}

struct WorkHostInner {
    started: bool,
    next_id: u64,
    runs: HashMap<WorkId, WorkRun>,
    queue: VecDeque<PendingWork>,
}

/// Cloneable process-owned host for live work handles.
///
/// Rust Hara values are currently evaluator-thread values (`Rc`, not `Send`).
/// The host therefore schedules work cooperatively on that evaluator thread.
/// Submission only enqueues; polling or waiting on the result Promise, or an
/// explicit [`WorkHost::run`], advances the run.
#[derive(Clone)]
pub struct WorkHost {
    inner: Rc<RefCell<WorkHostInner>>,
}

impl fmt::Debug for WorkHost {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WorkHost")
            .field("status", &self.status())
            .finish()
    }
}

impl Default for WorkHost {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkHost {
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(WorkHostInner {
                started: true,
                next_id: 1,
                runs: HashMap::new(),
                queue: VecDeque::new(),
            })),
        }
    }

    /// Submit work without executing it inline.
    pub fn submit<F>(&self, id: Option<&str>, task: F) -> Result<WorkRun, String>
    where
        F: FnOnce() -> Result<Value, String> + 'static,
    {
        self.submit_rejection(id, move || task().map_err(work_failure))
    }

    /// Submit work whose executor already returns a native structured rejection.
    pub fn submit_rejection<F>(&self, id: Option<&str>, task: F) -> Result<WorkRun, String>
    where
        F: FnOnce() -> Result<Value, PromiseRejection> + 'static,
    {
        let mut host = self.inner.borrow_mut();
        if !host.started {
            return Err("native work host is stopped".into());
        }
        let id = match id {
            Some(id) => WorkId::new(id)?,
            None => next_work_id(&mut host)?,
        };
        if host.runs.contains_key(&id) {
            return Err(format!("work run ID already exists: {id}"));
        }

        let result = Promise::new();
        let run = WorkRun {
            inner: Rc::new(WorkRunInner {
                id: id.clone(),
                result,
                status: RefCell::new(WorkRunStatus {
                    id: id.clone(),
                    state: WorkRunState::Queued,
                    started_at_millis: now_millis(),
                    finished_at_millis: None,
                    error: None,
                }),
                host: Rc::downgrade(&self.inner),
            }),
        };
        install_progress_hooks(self, &run);
        host.runs.insert(id.clone(), run.clone());
        host.queue.push_back(PendingWork {
            id,
            task: Box::new(task),
        });
        Ok(run)
    }

    /// Resolve a live handle from a portable raw identifier.
    pub fn resolve(&self, reference: &str) -> Result<WorkRun, String> {
        let id = WorkId::new(reference)?;
        self.resolve_id(&id)
    }

    pub fn resolve_id(&self, id: &WorkId) -> Result<WorkRun, String> {
        self.inner
            .borrow()
            .runs
            .get(id)
            .cloned()
            .ok_or_else(|| format!("unknown work run: {id}"))
    }

    /// Run one queued item by ID. Returns false when no runnable item remains.
    pub fn run(&self, id: &WorkId) -> bool {
        let (run, task) = {
            let mut host = self.inner.borrow_mut();
            let Some(index) = host.queue.iter().position(|pending| &pending.id == id) else {
                return false;
            };
            let pending = host.queue.remove(index).expect("queued work disappeared");
            let run = host
                .runs
                .get(id)
                .cloned()
                .expect("queued work has no live run");
            (run, pending.task)
        };
        if !run.mark_running() {
            return false;
        }
        match task() {
            Ok(value) => run.settle_value(value),
            Err(error) => run.fail(error),
        }
        true
    }

    pub fn run_next(&self) -> bool {
        let id = self
            .inner
            .borrow()
            .queue
            .front()
            .map(|pending| pending.id.clone());
        id.is_some_and(|id| self.run(&id))
    }

    pub fn drain(&self) {
        while self.run_next() {}
    }

    pub fn status(&self) -> WorkHostStatus {
        let host = self.inner.borrow();
        WorkHostStatus {
            state: if host.started { "started" } else { "stopped" },
            run_count: host.runs.len(),
            queued_count: host.queue.len(),
        }
    }

    pub fn started(&self) -> bool {
        self.inner.borrow().started
    }

    pub fn start(&self) {
        self.inner.borrow_mut().started = true;
    }

    pub fn stop(&self) {
        let runs = {
            let mut host = self.inner.borrow_mut();
            host.started = false;
            host.runs.values().cloned().collect::<Vec<_>>()
        };
        for run in runs {
            run.cancel(Value::Keyword("host-stopped".into()));
        }
    }

    pub fn same_identity(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

impl IComponent for WorkHost {
    type Metadata = WorkHostStatus;

    fn props(&self) -> Self::Metadata {
        self.status()
    }

    fn status(&self) -> Self::Metadata {
        WorkHost::status(self)
    }

    fn started(&self) -> bool {
        WorkHost::started(self)
    }

    fn stopped(&self) -> bool {
        !WorkHost::started(self)
    }

    fn start(&mut self) {
        WorkHost::start(self);
    }

    fn stop(&mut self) {
        WorkHost::stop(self);
    }
}

struct WorkRunInner {
    id: WorkId,
    result: Promise,
    status: RefCell<WorkRunStatus>,
    host: Weak<RefCell<WorkHostInner>>,
}

/// Live process-owned handle returned immediately from submission.
#[derive(Clone)]
pub struct WorkRun {
    inner: Rc<WorkRunInner>,
}

impl fmt::Debug for WorkRun {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WorkRun")
            .field("status", &self.work_status())
            .finish()
    }
}

impl WorkRun {
    pub fn work_id(&self) -> WorkId {
        self.inner.id.clone()
    }

    pub fn work_status(&self) -> WorkRunStatus {
        self.inner.status.borrow().clone()
    }

    /// Return the same native result Promise on every call.
    pub fn work_result(&self) -> Promise {
        self.inner.result.clone()
    }

    pub fn work_cancel(&self, reason: Value) -> Promise {
        let result = Promise::new();
        result.resolve(Value::Bool(self.cancel(reason)));
        result
    }

    pub fn cancel(&self, reason: Value) -> bool {
        let rejection = cancellation_rejection(reason);
        if !self.transition_terminal(WorkRunState::Cancelled, Some(rejection.clone())) {
            return false;
        }
        if let Some(host) = self.inner.host.upgrade() {
            host.borrow_mut()
                .queue
                .retain(|pending| pending.id != self.inner.id);
        }
        self.inner.result.reject_rejection(rejection);
        true
    }

    pub fn closed(&self) -> bool {
        self.inner.status.borrow().state.terminal()
    }

    pub fn same_identity(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }

    fn mark_running(&self) -> bool {
        let mut status = self.inner.status.borrow_mut();
        if status.state != WorkRunState::Queued {
            return false;
        }
        status.state = WorkRunState::Running;
        true
    }

    fn settle_value(&self, value: Value) {
        if let Value::Promise(source) = value {
            self.adopt(source);
            return;
        }
        if self.transition_terminal(WorkRunState::Completed, None) {
            self.inner.result.resolve(value);
        }
    }

    fn adopt(&self, source: Promise) {
        if source.same_identity(&self.inner.result) {
            self.fail(work_failure("work result promise adoption cycle".into()));
            return;
        }
        let run = Rc::downgrade(&self.inner);
        source.on_settle(Rc::new(move |state| {
            let Some(inner) = run.upgrade() else {
                return;
            };
            let run = WorkRun { inner };
            match state {
                crate::core::PromiseState::Pending => {}
                crate::core::PromiseState::Fulfilled(_) => {
                    run.transition_terminal(WorkRunState::Completed, None);
                }
                crate::core::PromiseState::Rejected(error) => {
                    run.transition_terminal(WorkRunState::Failed, Some(error));
                }
            }
        }));
        self.inner.result.adopt(&source);
    }

    fn fail(&self, error: PromiseRejection) {
        if self.transition_terminal(WorkRunState::Failed, Some(error.clone())) {
            self.inner.result.reject_rejection(error);
        }
    }

    fn transition_terminal(
        &self,
        state: WorkRunState,
        error: Option<PromiseRejection>,
    ) -> bool {
        let mut status = self.inner.status.borrow_mut();
        if status.state.terminal() {
            return false;
        }
        status.state = state;
        status.finished_at_millis = Some(now_millis());
        status.error = error;
        true
    }
}

thread_local! {
    static PROCESS_WORK_HOST: WorkHost = WorkHost::new();
}

/// Return the process/evaluator-thread host shared by independent sessions.
pub fn process_work_host() -> WorkHost {
    PROCESS_WORK_HOST.with(Clone::clone)
}

fn install_progress_hooks(host: &WorkHost, run: &WorkRun) {
    let weak_host = Rc::downgrade(&host.inner);
    let id = run.work_id();
    run.inner.result.set_poller(Rc::new(move || {
        if let Some(inner) = weak_host.upgrade() {
            WorkHost { inner }.run(&id);
        }
    }));

    let weak_host = Rc::downgrade(&host.inner);
    let id = run.work_id();
    run.inner.result.set_waiter(Rc::new(move || {
        if let Some(inner) = weak_host.upgrade() {
            WorkHost { inner }.run(&id);
        }
    }));
}

fn next_work_id(host: &mut WorkHostInner) -> Result<WorkId, String> {
    loop {
        let id = WorkId(format!("run-{}", host.next_id));
        host.next_id = host
            .next_id
            .checked_add(1)
            .ok_or_else(|| "work run identifiers exhausted".to_string())?;
        if !host.runs.contains_key(&id) {
            return Ok(id);
        }
    }
}

fn work_failure(message: String) -> PromiseRejection {
    PromiseRejection::Value(Value::Map(
        [
            (
                Value::Keyword("code".into()),
                Value::Keyword("work/failed".into()),
            ),
            (
                Value::Keyword("message".into()),
                Value::String(message),
            ),
            (Value::Keyword("retryable".into()), Value::Bool(false)),
        ]
        .into_iter()
        .collect(),
    ))
}

fn cancellation_rejection(reason: Value) -> PromiseRejection {
    PromiseRejection::Cancelled(Value::Map(
        [
            (
                Value::Keyword("code".into()),
                Value::Keyword("work/cancelled".into()),
            ),
            (Value::Keyword("reason".into()), reason),
            (Value::Keyword("retryable".into()), Value::Bool(false)),
        ]
        .into_iter()
        .collect(),
    ))
}

fn now_millis() -> u64 {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    u64::try_from(millis).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::PromiseState;
    use std::cell::Cell;

    #[test]
    fn submission_returns_a_queued_handle_before_execution() {
        let host = WorkHost::new();
        let executed = Rc::new(Cell::new(false));
        let task_executed = executed.clone();
        let run = host
            .submit(Some("immediate"), move || {
                task_executed.set(true);
                Ok(Value::Number(7))
            })
            .unwrap();

        assert_eq!(run.work_status().state, WorkRunState::Queued);
        assert!(!executed.get());
        assert_eq!(
            run.work_result().state(),
            PromiseState::Fulfilled(Value::Number(7))
        );
        assert!(executed.get());
        assert_eq!(run.work_status().state, WorkRunState::Completed);
    }

    #[test]
    fn independent_session_kernels_resolve_the_same_live_handle() {
        let first = crate::SessionKernel::new().work_host();
        let second = crate::SessionKernel::new().work_host();
        assert!(first.same_identity(&second));

        let run = first
            .submit(None, || Ok(Value::String("shared".into())))
            .unwrap();
        let resolved = second.resolve_id(&run.work_id()).unwrap();
        assert!(run.same_identity(&resolved));
        assert!(run
            .work_result()
            .same_identity(&resolved.work_result()));
        assert_eq!(
            resolved.work_result().state(),
            PromiseState::Fulfilled(Value::String("shared".into()))
        );
    }

    #[test]
    fn failure_is_retained_and_rejects_the_cached_result() {
        let host = WorkHost::new();
        let run = host
            .submit(Some("failed"), || Err("executor failed".into()))
            .unwrap();
        let result = run.work_result();
        assert!(result.same_identity(&run.work_result()));
        assert!(matches!(result.state(), PromiseState::Rejected(_)));

        let status = run.work_status();
        assert_eq!(status.state, WorkRunState::Failed);
        let Some(PromiseRejection::Value(Value::Map(fields))) = status.error else {
            panic!("work failure was not retained as a structured value");
        };
        assert_eq!(
            fields.get(&Value::Keyword("code".into())),
            Some(&Value::Keyword("work/failed".into()))
        );
    }

    #[test]
    fn cancelling_queued_work_prevents_its_executor_from_starting() {
        let host = WorkHost::new();
        let executed = Rc::new(Cell::new(false));
        let task_executed = executed.clone();
        let run = host
            .submit(Some("cancelled"), move || {
                task_executed.set(true);
                Ok(Value::Number(1))
            })
            .unwrap();

        assert!(run.cancel(Value::Keyword("test".into())));
        host.drain();
        assert!(!executed.get());
        assert_eq!(run.work_status().state, WorkRunState::Cancelled);
        assert!(matches!(
            run.work_result().state(),
            PromiseState::Rejected(ref error) if error.is_cancelled()
        ));
    }

    #[test]
    fn adopted_promises_settle_status_and_result_once() {
        let host = WorkHost::new();
        let source = Promise::new();
        let task_source = source.clone();
        let run = host
            .submit(Some("adopted"), move || Ok(Value::Promise(task_source)))
            .unwrap();
        let result = run.work_result();

        assert_eq!(result.state(), PromiseState::Pending);
        assert_eq!(run.work_status().state, WorkRunState::Running);
        source.resolve(Value::Number(42));
        assert_eq!(run.work_status().state, WorkRunState::Completed);
        assert_eq!(result.state(), PromiseState::Fulfilled(Value::Number(42)));
        assert!(!run.cancel(Value::Keyword("late".into())));
        assert_eq!(result.state(), PromiseState::Fulfilled(Value::Number(42)));
    }
}
