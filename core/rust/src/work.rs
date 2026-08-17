use crate::core::{Promise, PromiseRejection, PromiseState, Value};
use crate::lang::protocol::IComponent;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::rc::{Rc, Weak};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

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
    Waiting,
    Cancelling,
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
    pub cancel_reason: Option<Value>,
    pub parent_id: Option<WorkId>,
    pub child_count: usize,
    pub deadline_remaining_millis: Option<u64>,
    pub detached: bool,
}

/// Process-host lifecycle metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkHostStatus {
    pub state: &'static str,
    pub run_count: usize,
    pub queued_count: usize,
}

/// Submission-time scope and deadline options.
#[derive(Clone, Debug, Default)]
pub struct WorkOptions {
    pub id: Option<WorkId>,
    pub timeout: Option<Duration>,
    pub deadline: Option<Instant>,
    pub detached: bool,
}

impl WorkOptions {
    pub fn with_id(id: impl Into<String>) -> Result<Self, String> {
        Ok(Self {
            id: Some(WorkId::new(id)?),
            ..Self::default()
        })
    }
}

type WorkTask = Box<dyn FnOnce(WorkContext) -> Result<Value, PromiseRejection>>;
type WorkFinalizer = Box<dyn FnOnce(WorkContext) -> Result<(), PromiseRejection>>;

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
        let options = WorkOptions {
            id: id.map(WorkId::new).transpose()?,
            ..WorkOptions::default()
        };
        self.submit_scoped(options, move |_| task())
    }

    /// Submit work whose executor already returns a native structured rejection.
    pub fn submit_rejection<F>(&self, id: Option<&str>, task: F) -> Result<WorkRun, String>
    where
        F: FnOnce() -> Result<Value, PromiseRejection> + 'static,
    {
        let options = WorkOptions {
            id: id.map(WorkId::new).transpose()?,
            ..WorkOptions::default()
        };
        self.submit_scoped_rejection(options, move |_| task())
    }

    pub fn submit_scoped<F>(&self, options: WorkOptions, task: F) -> Result<WorkRun, String>
    where
        F: FnOnce(WorkContext) -> Result<Value, String> + 'static,
    {
        self.submit_scoped_rejection(options, move |context| task(context).map_err(work_failure))
    }

    pub fn submit_scoped_rejection<F>(
        &self,
        options: WorkOptions,
        task: F,
    ) -> Result<WorkRun, String>
    where
        F: FnOnce(WorkContext) -> Result<Value, PromiseRejection> + 'static,
    {
        let parent = if options.detached {
            None
        } else {
            current_work_context()
                .filter(|context| context.host.same_identity(self))
                .map(|context| context.run)
        };
        self.submit_with_parent(parent, options, Box::new(task))
    }

    fn submit_with_parent(
        &self,
        parent: Option<WorkRun>,
        options: WorkOptions,
        task: WorkTask,
    ) -> Result<WorkRun, String> {
        let mut host = self.inner.borrow_mut();
        if !host.started {
            return Err("native work host is stopped".into());
        }
        let deadline = resolve_deadline(&options, parent.as_ref());
        let id = match options.id.clone() {
            Some(id) => id,
            None => next_work_id(&mut host)?,
        };
        if host.runs.contains_key(&id) {
            return Err(format!("work run ID already exists: {id}"));
        }
        if parent
            .as_ref()
            .is_some_and(|parent| !parent.accepts_children())
        {
            return Err("parent work scope is closed".into());
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
                    cancel_reason: None,
                    parent_id: parent.as_ref().map(WorkRun::work_id),
                    child_count: 0,
                    deadline_remaining_millis: deadline.map(deadline_remaining_millis),
                    detached: options.detached,
                }),
                host: Rc::downgrade(&self.inner),
                parent: parent.as_ref().map(|parent| Rc::downgrade(&parent.inner)),
                children: RefCell::new(HashMap::new()),
                deadline,
                cancellation: RefCell::new(None),
                body_done: Cell::new(false),
                body_outcome: RefCell::new(None),
                finalizers: RefCell::new(Vec::new()),
                finalizers_started: Cell::new(false),
                active_promise: RefCell::new(None),
                parent_notified: Cell::new(false),
            }),
        };
        if let Some(parent) = &parent {
            if !parent.attach_child(run.clone()) {
                return Err("parent work scope is closed".into());
            }
        }
        install_progress_hooks(self, &run);
        host.runs.insert(id.clone(), run.clone());
        host.queue.push_back(PendingWork { id, task });
        drop(host);
        run.check_deadline();
        Ok(run)
    }

    /// Resolve a live handle from a portable raw identifier.
    pub fn resolve(&self, reference: &str) -> Result<WorkRun, String> {
        let id = WorkId::new(reference)?;
        self.resolve_id(&id)
    }

    pub fn resolve_id(&self, id: &WorkId) -> Result<WorkRun, String> {
        let run = self
            .inner
            .borrow()
            .runs
            .get(id)
            .cloned()
            .ok_or_else(|| format!("unknown work run: {id}"))?;
        run.check_deadline();
        Ok(run)
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
        run.check_deadline();
        if !run.mark_running() {
            return false;
        }
        let context = WorkContext {
            host: self.clone(),
            run: run.clone(),
        };
        let result = with_current_work_context(context.clone(), || task(context));
        match result {
            Ok(value) => run.settle_body(value),
            Err(error) => run.fail_body(error),
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

    fn progress(&self, id: &WorkId) {
        let _ = self.run(id);
        let run = self.resolve_id(id).ok();
        if let Some(run) = &run {
            run.progress_active_promise();
        }
        while run.as_ref().is_some_and(|run| !run.closed()) && self.run_next() {}
        if let Some(run) = run {
            run.progress_active_promise();
            run.check_deadline();
        }
    }

    fn wait_for(&self, id: &WorkId) {
        self.progress(id);
        if let Ok(run) = self.resolve_id(id) {
            let active = run.inner.active_promise.borrow().clone();
            if let Some(active) = active {
                active.wait_state();
            }
            self.progress(id);
        }
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

#[derive(Clone)]
struct CancellationRequest {
    reason: Value,
    rejection: PromiseRejection,
}

struct WorkRunInner {
    id: WorkId,
    result: Promise,
    status: RefCell<WorkRunStatus>,
    host: Weak<RefCell<WorkHostInner>>,
    parent: Option<Weak<WorkRunInner>>,
    children: RefCell<HashMap<WorkId, WorkRun>>,
    deadline: Option<Instant>,
    cancellation: RefCell<Option<CancellationRequest>>,
    body_done: Cell<bool>,
    body_outcome: RefCell<Option<Result<Value, PromiseRejection>>>,
    finalizers: RefCell<Vec<WorkFinalizer>>,
    finalizers_started: Cell<bool>,
    active_promise: RefCell<Option<Promise>>,
    parent_notified: Cell<bool>,
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
        self.check_deadline();
        let mut status = self.inner.status.borrow().clone();
        status.child_count = self.inner.children.borrow().len();
        status.deadline_remaining_millis = self.inner.deadline.map(deadline_remaining_millis);
        status
    }

    pub fn deadline(&self) -> Option<Instant> {
        self.inner.deadline
    }

    pub fn cancellation_token(&self) -> WorkCancellationToken {
        WorkCancellationToken {
            run: Rc::downgrade(&self.inner),
        }
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
        let rejection = cancellation_rejection(reason.clone());
        {
            let mut cancellation = self.inner.cancellation.borrow_mut();
            if cancellation.is_some() || self.closed() {
                return false;
            }
            *cancellation = Some(CancellationRequest {
                reason: reason.clone(),
                rejection: rejection.clone(),
            });
        }

        let previous = {
            let mut status = self.inner.status.borrow_mut();
            let previous = status.state;
            if !previous.terminal() {
                status.state = WorkRunState::Cancelling;
                status.cancel_reason = Some(reason.clone());
            }
            previous
        };
        if previous == WorkRunState::Queued {
            if let Some(host) = self.inner.host.upgrade() {
                host.borrow_mut()
                    .queue
                    .retain(|pending| pending.id != self.inner.id);
            }
            self.inner.body_done.set(true);
        }
        let active = self.inner.active_promise.borrow().clone();
        if let Some(active) = active {
            active.cancel();
        }
        let children = self
            .inner
            .children
            .borrow()
            .values()
            .cloned()
            .collect::<Vec<_>>();
        for child in children {
            child.cancel(reason.clone());
        }
        self.finish_if_ready();
        true
    }

    pub fn closed(&self) -> bool {
        self.inner.status.borrow().state.terminal()
    }

    pub fn same_identity(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }

    fn accepts_children(&self) -> bool {
        !self.inner.body_done.get()
            && !self.inner.finalizers_started.get()
            && self.inner.cancellation.borrow().is_none()
            && !self.closed()
    }

    fn attach_child(&self, child: WorkRun) -> bool {
        if !self.accepts_children() {
            return false;
        }
        self.inner
            .children
            .borrow_mut()
            .insert(child.work_id(), child);
        true
    }

    fn child_closed(&self, child: &WorkRun) {
        self.inner.children.borrow_mut().remove(&child.work_id());
        self.finish_if_ready();
    }

    fn mark_running(&self) -> bool {
        self.check_deadline();
        let mut status = self.inner.status.borrow_mut();
        if status.state != WorkRunState::Queued {
            return false;
        }
        status.state = WorkRunState::Running;
        true
    }

    fn settle_body(&self, value: Value) {
        if let Value::Promise(source) = value {
            if source.same_identity(&self.inner.result) {
                self.fail_body(work_failure("work result promise adoption cycle".into()));
                return;
            }
            *self.inner.active_promise.borrow_mut() = Some(source.clone());
            if self.inner.cancellation.borrow().is_some() {
                source.cancel();
            }
            let run = Rc::downgrade(&self.inner);
            source.on_settle(Rc::new(move |state| {
                let Some(inner) = run.upgrade() else {
                    return;
                };
                let run = WorkRun { inner };
                run.inner.active_promise.borrow_mut().take();
                match state {
                    PromiseState::Pending => return,
                    PromiseState::Fulfilled(value) => {
                        *run.inner.body_outcome.borrow_mut() = Some(Ok(value));
                    }
                    PromiseState::Rejected(error) => {
                        *run.inner.body_outcome.borrow_mut() = Some(Err(error));
                    }
                }
                run.inner.body_done.set(true);
                run.finish_if_ready();
            }));
            self.set_nonterminal_state(if self.inner.cancellation.borrow().is_some() {
                WorkRunState::Cancelling
            } else {
                WorkRunState::Waiting
            });
            source.state();
            return;
        }
        *self.inner.body_outcome.borrow_mut() = Some(Ok(value));
        self.inner.body_done.set(true);
        self.finish_if_ready();
    }

    fn fail_body(&self, error: PromiseRejection) {
        *self.inner.body_outcome.borrow_mut() = Some(Err(error));
        self.inner.body_done.set(true);
        self.finish_if_ready();
    }

    fn progress_active_promise(&self) {
        let active = self.inner.active_promise.borrow().clone();
        if let Some(active) = active {
            active.state();
        }
    }

    fn check_deadline(&self) {
        if self
            .inner
            .deadline
            .is_some_and(|deadline| Instant::now() >= deadline)
        {
            self.cancel(Value::Keyword("deadline-exceeded".into()));
        }
    }

    fn register_finalizer(&self, finalizer: WorkFinalizer) -> bool {
        if self.inner.finalizers_started.get() || self.closed() {
            return false;
        }
        self.inner.finalizers.borrow_mut().push(finalizer);
        true
    }

    fn finish_if_ready(&self) {
        if !self.inner.body_done.get() {
            return;
        }
        if !self.inner.children.borrow().is_empty() {
            self.set_nonterminal_state(if self.inner.cancellation.borrow().is_some() {
                WorkRunState::Cancelling
            } else {
                WorkRunState::Waiting
            });
            return;
        }
        if self.inner.finalizers_started.replace(true) {
            return;
        }

        let context = self.context();
        let mut finalizer_error = None;
        let mut finalizers = std::mem::take(&mut *self.inner.finalizers.borrow_mut());
        while let Some(finalizer) = finalizers.pop() {
            let result = with_current_work_context(context.clone(), || finalizer(context.clone()));
            if finalizer_error.is_none() {
                if let Err(error) = result {
                    finalizer_error = Some(error);
                }
            }
        }

        if let Some(cancellation) = self.inner.cancellation.borrow().clone() {
            self.settle_terminal(
                WorkRunState::Cancelled,
                Some(cancellation.rejection.clone()),
                Some(cancellation.reason),
                None,
            );
            return;
        }
        if let Some(error) = finalizer_error {
            self.settle_terminal(WorkRunState::Failed, Some(error), None, None);
            return;
        }
        match self.inner.body_outcome.borrow_mut().take() {
            Some(Ok(value)) => {
                self.settle_terminal(WorkRunState::Completed, None, None, Some(value));
            }
            Some(Err(error)) => {
                self.settle_terminal(WorkRunState::Failed, Some(error), None, None);
            }
            None => {
                self.settle_terminal(
                    WorkRunState::Failed,
                    Some(work_failure("work body produced no outcome".into())),
                    None,
                    None,
                );
            }
        }
    }

    fn settle_terminal(
        &self,
        state: WorkRunState,
        error: Option<PromiseRejection>,
        cancel_reason: Option<Value>,
        value: Option<Value>,
    ) -> bool {
        {
            let mut status = self.inner.status.borrow_mut();
            if status.state.terminal() {
                return false;
            }
            status.state = state;
            status.finished_at_millis = Some(now_millis());
            status.error = error.clone();
            status.cancel_reason = cancel_reason;
        }
        match state {
            WorkRunState::Completed => {
                self.inner.result.resolve(value.unwrap_or(Value::Nil));
            }
            WorkRunState::Failed | WorkRunState::Cancelled => {
                self.inner
                    .result
                    .reject_rejection(error.expect("terminal failure requires rejection"));
            }
            _ => unreachable!("non-terminal state passed to settle_terminal"),
        }
        self.notify_parent();
        true
    }

    fn set_nonterminal_state(&self, state: WorkRunState) {
        let mut status = self.inner.status.borrow_mut();
        if !status.state.terminal() {
            status.state = state;
        }
    }

    fn notify_parent(&self) {
        if self.inner.parent_notified.replace(true) {
            return;
        }
        let Some(parent) = self.inner.parent.as_ref().and_then(Weak::upgrade) else {
            return;
        };
        WorkRun { inner: parent }.child_closed(self);
    }

    fn context(&self) -> WorkContext {
        let host = self
            .inner
            .host
            .upgrade()
            .map(|inner| WorkHost { inner })
            .expect("work host was dropped while run remained live");
        WorkContext {
            host,
            run: self.clone(),
        }
    }
}

/// Cooperative cancellation token exposed through the active work context.
#[derive(Clone)]
pub struct WorkCancellationToken {
    run: Weak<WorkRunInner>,
}

impl WorkCancellationToken {
    pub fn cancelled(&self) -> bool {
        self.run
            .upgrade()
            .is_none_or(|run| run.cancellation.borrow().is_some())
    }

    pub fn reason(&self) -> Option<Value> {
        self.run.upgrade().and_then(|run| {
            run.cancellation
                .borrow()
                .as_ref()
                .map(|request| request.reason.clone())
        })
    }

    pub fn check(&self) -> Result<(), PromiseRejection> {
        let Some(run) = self.run.upgrade() else {
            return Err(cancellation_rejection(Value::Keyword(
                "scope-closed".into(),
            )));
        };
        let run = WorkRun { inner: run };
        run.check_deadline();
        match run.inner.cancellation.borrow().as_ref() {
            Some(request) => Err(request.rejection.clone()),
            None => Ok(()),
        }
    }
}

/// Opaque evaluator-thread context for one native work scope.
#[derive(Clone)]
pub struct WorkContext {
    host: WorkHost,
    run: WorkRun,
}

impl WorkContext {
    pub fn work_id(&self) -> WorkId {
        self.run.work_id()
    }

    pub fn token(&self) -> WorkCancellationToken {
        self.run.cancellation_token()
    }

    pub fn cancelled(&self) -> bool {
        self.token().cancelled()
    }

    pub fn cancel_reason(&self) -> Option<Value> {
        self.token().reason()
    }

    pub fn deadline(&self) -> Option<Instant> {
        self.run.deadline()
    }

    pub fn check_cancelled(&self) -> Result<(), PromiseRejection> {
        self.token().check()
    }

    pub fn submit_child<F>(&self, options: WorkOptions, task: F) -> Result<WorkRun, String>
    where
        F: FnOnce(WorkContext) -> Result<Value, String> + 'static,
    {
        self.check_cancelled().map_err(|error| error.message())?;
        let parent = if options.detached {
            None
        } else {
            Some(self.run.clone())
        };
        self.host.submit_with_parent(
            parent,
            options,
            Box::new(move |context| task(context).map_err(work_failure)),
        )
    }

    pub fn submit_child_rejection<F>(
        &self,
        options: WorkOptions,
        task: F,
    ) -> Result<WorkRun, String>
    where
        F: FnOnce(WorkContext) -> Result<Value, PromiseRejection> + 'static,
    {
        self.check_cancelled().map_err(|error| error.message())?;
        let parent = if options.detached {
            None
        } else {
            Some(self.run.clone())
        };
        self.host
            .submit_with_parent(parent, options, Box::new(task))
    }

    pub fn on_close<F>(&self, finalizer: F) -> bool
    where
        F: FnOnce(WorkContext) -> Result<(), PromiseRejection> + 'static,
    {
        self.run.register_finalizer(Box::new(finalizer))
    }
}

thread_local! {
    static PROCESS_WORK_HOST: WorkHost = WorkHost::new();
    static CURRENT_WORK_CONTEXT: RefCell<Option<WorkContext>> = const { RefCell::new(None) };
}

/// Return the process/evaluator-thread host shared by independent sessions.
pub fn process_work_host() -> WorkHost {
    PROCESS_WORK_HOST.with(Clone::clone)
}

/// Return the currently executing cooperative work context, if any.
pub fn current_work_context() -> Option<WorkContext> {
    CURRENT_WORK_CONTEXT.with(|current| current.borrow().clone())
}

fn with_current_work_context<T>(context: WorkContext, function: impl FnOnce() -> T) -> T {
    let previous = CURRENT_WORK_CONTEXT.with(|current| current.replace(Some(context)));
    let result = function();
    CURRENT_WORK_CONTEXT.with(|current| {
        current.replace(previous);
    });
    result
}

fn install_progress_hooks(host: &WorkHost, run: &WorkRun) {
    let weak_host = Rc::downgrade(&host.inner);
    let id = run.work_id();
    run.inner.result.set_poller(Rc::new(move || {
        if let Some(inner) = weak_host.upgrade() {
            WorkHost { inner }.progress(&id);
        }
    }));

    let weak_host = Rc::downgrade(&host.inner);
    let id = run.work_id();
    run.inner.result.set_waiter(Rc::new(move || {
        if let Some(inner) = weak_host.upgrade() {
            WorkHost { inner }.wait_for(&id);
        }
    }));

    let weak_run = Rc::downgrade(&run.inner);
    run.inner.result.set_cancel_hook(Rc::new(move || {
        if let Some(inner) = weak_run.upgrade() {
            WorkRun { inner }.cancel(Value::Keyword("result-cancelled".into()));
        }
    }));
}

fn resolve_deadline(options: &WorkOptions, parent: Option<&WorkRun>) -> Option<Instant> {
    let inherited = parent.and_then(WorkRun::deadline);
    let relative = options
        .timeout
        .and_then(|timeout| Instant::now().checked_add(timeout));
    [inherited, options.deadline, relative]
        .into_iter()
        .flatten()
        .min()
}

fn deadline_remaining_millis(deadline: Instant) -> u64 {
    u64::try_from(
        deadline
            .saturating_duration_since(Instant::now())
            .as_millis(),
    )
    .unwrap_or(u64::MAX)
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
            (Value::Keyword("message".into()), Value::String(message)),
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
        assert!(run.work_result().same_identity(&resolved.work_result()));
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
        assert_eq!(run.work_status().state, WorkRunState::Waiting);
        source.resolve(Value::Number(42));
        assert_eq!(run.work_status().state, WorkRunState::Completed);
        assert_eq!(result.state(), PromiseState::Fulfilled(Value::Number(42)));
        assert!(!run.cancel(Value::Keyword("late".into())));
        assert_eq!(result.state(), PromiseState::Fulfilled(Value::Number(42)));
    }

    #[test]
    fn cancellation_cancels_active_promise_and_runs_finalizers_once() {
        let host = WorkHost::new();
        let source = Promise::new();
        let task_source = source.clone();
        let cleanups = Rc::new(Cell::new(0));
        let task_cleanups = cleanups.clone();
        let run = host
            .submit_scoped(
                WorkOptions::with_id("scope-cancel").unwrap(),
                move |context| {
                    context.on_close(move |_| {
                        task_cleanups.set(task_cleanups.get() + 1);
                        Ok(())
                    });
                    Ok(Value::Promise(task_source))
                },
            )
            .unwrap();
        host.run(&run.work_id());
        assert_eq!(run.work_status().state, WorkRunState::Waiting);

        assert!(run.cancel(Value::Keyword("stop".into())));
        assert!(!run.cancel(Value::Keyword("again".into())));
        assert_eq!(cleanups.get(), 1);
        assert_eq!(run.work_status().state, WorkRunState::Cancelled);
        assert!(matches!(source.state(), PromiseState::Rejected(_)));
    }

    #[test]
    fn parent_waits_for_attached_child_and_cancellation_flows_downward() {
        let host = WorkHost::new();
        let child_source = Promise::new();
        let task_child_source = child_source.clone();
        let parent = host
            .submit_scoped(WorkOptions::with_id("parent").unwrap(), move |context| {
                context.submit_child(WorkOptions::with_id("child").unwrap(), move |_| {
                    Ok(Value::Promise(task_child_source))
                })?;
                Ok(Value::Keyword("parent".into()))
            })
            .unwrap();
        host.run(&parent.work_id());
        host.run_next();
        assert_eq!(parent.work_status().state, WorkRunState::Waiting);
        let child = host.resolve("child").unwrap();
        assert_eq!(child.work_status().parent_id, Some(parent.work_id()));

        assert!(parent.cancel(Value::Keyword("parent-stop".into())));
        assert_eq!(parent.work_status().state, WorkRunState::Cancelled);
        assert_eq!(child.work_status().state, WorkRunState::Cancelled);
        assert!(matches!(child_source.state(), PromiseState::Rejected(_)));
    }

    #[test]
    fn parent_completion_is_released_after_child_completion() {
        let host = WorkHost::new();
        let child_source = Promise::new();
        let task_child_source = child_source.clone();
        let parent = host
            .submit_scoped(
                WorkOptions::with_id("wait-parent").unwrap(),
                move |context| {
                    context
                        .submit_child(WorkOptions::with_id("wait-child").unwrap(), move |_| {
                            Ok(Value::Promise(task_child_source))
                        })?;
                    Ok(Value::Keyword("parent".into()))
                },
            )
            .unwrap();
        host.run(&parent.work_id());
        host.run_next();
        assert_eq!(parent.work_status().state, WorkRunState::Waiting);
        assert_eq!(parent.work_result().state(), PromiseState::Pending);

        child_source.resolve(Value::Keyword("child".into()));
        assert_eq!(parent.work_status().state, WorkRunState::Completed);
        assert_eq!(
            parent.work_result().state(),
            PromiseState::Fulfilled(Value::Keyword("parent".into()))
        );
    }

    #[test]
    fn deadlines_are_inherited_and_cancel_at_cooperative_safe_points() {
        let host = WorkHost::new();
        let child_source = Promise::new();
        let task_child_source = child_source.clone();
        let parent = host
            .submit_scoped(
                WorkOptions {
                    id: Some(WorkId::new("deadline-parent").unwrap()),
                    timeout: Some(Duration::from_millis(5)),
                    ..WorkOptions::default()
                },
                move |context| {
                    context.submit_child(
                        WorkOptions {
                            id: Some(WorkId::new("deadline-child").unwrap()),
                            timeout: Some(Duration::from_secs(1)),
                            ..WorkOptions::default()
                        },
                        move |_| Ok(Value::Promise(task_child_source)),
                    )?;
                    Ok(Value::Promise(Promise::new()))
                },
            )
            .unwrap();
        host.run(&parent.work_id());
        host.run_next();
        let child = host.resolve("deadline-child").unwrap();
        assert_eq!(parent.deadline(), child.deadline());

        std::thread::sleep(Duration::from_millis(8));
        parent.work_status();
        assert_eq!(parent.work_status().state, WorkRunState::Cancelled);
        assert_eq!(child.work_status().state, WorkRunState::Cancelled);
    }

    #[test]
    fn detached_children_outlive_parent_cancellation() {
        let host = WorkHost::new();
        let parent = host
            .submit_scoped(
                WorkOptions::with_id("detach-parent").unwrap(),
                move |context| {
                    context.submit_child(
                        WorkOptions {
                            id: Some(WorkId::new("detached-child").unwrap()),
                            detached: true,
                            ..WorkOptions::default()
                        },
                        move |_| Ok(Value::Number(42)),
                    )?;
                    Ok(Value::Promise(Promise::new()))
                },
            )
            .unwrap();
        host.run(&parent.work_id());
        let child = host.resolve("detached-child").unwrap();
        assert!(child.work_status().detached);
        assert_eq!(child.work_status().parent_id, None);

        assert!(parent.cancel(Value::Keyword("stop-parent".into())));
        assert_eq!(child.work_status().state, WorkRunState::Queued);
        host.run(&child.work_id());
        assert_eq!(child.work_status().state, WorkRunState::Completed);
    }
}
