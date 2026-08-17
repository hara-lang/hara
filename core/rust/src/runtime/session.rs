/// A process-local kernel that multiplexes isolated evaluator sessions.
///
/// Raw HTA exposes the same lifecycle over its wire targets; this native
/// facade keeps embedding hosts from treating a `Runtime` as the process
/// boundary when several independent sessions can share one kernel.
pub struct SessionKernel {
    sessions: HashMap<String, Session>,
    resources: HashMap<String, String>,
    mounts: HashMap<u64, FilesystemMount>,
    session_mounts: HashMap<String, u64>,
    next_mount_id: u64,
    test_runner: String,
}

/// An isolated, named execution context owned by a [`SessionKernel`].
pub struct Session {
    spec: SessionSpec,
    runtime: Option<Runtime>,
    state: SessionState,
    filesystem: Option<AttachedFilesystem>,
    authority: SessionAuthorityPolicy,
    last_namespace: String,
}

struct AttachedFilesystem {
    id: SessionMountId,
    _provider: Rc<dyn core::FileProvider>,
}

impl Session {
    fn new(name: &str, runtime: Runtime) -> Self {
        let spec = SessionSpec::zero_authority(name)
            .expect("Session::new requires a validated session name");
        Self::open(spec, runtime)
    }

    fn open(spec: SessionSpec, runtime: Runtime) -> Self {
        let authority = spec.authority;
        let mut session = Self {
            spec,
            runtime: Some(runtime),
            state: SessionState::New,
            filesystem: None,
            authority,
            last_namespace: "user".into(),
        };
        session.activate();
        session
    }

    pub fn spec(&self) -> &SessionSpec {
        &self.spec
    }

    pub fn id(&self) -> &SessionId {
        &self.spec.id
    }

    pub fn name(&self) -> &str {
        self.id().as_str()
    }

    pub fn state(&self) -> SessionState {
        self.state
    }

    pub fn filesystem_mount(&self) -> Option<SessionMountId> {
        self.filesystem.as_ref().map(|filesystem| filesystem.id)
    }

    fn ensure_active(&self) -> Result<(), String> {
        match self.state {
            SessionState::Active => Ok(()),
            SessionState::Closed => Err(format!("SESSION_CLOSED {}", self.name())),
            SessionState::New => Err(format!("SESSION_NOT_ACTIVE {} new", self.name())),
        }
    }

    fn runtime_mut(&mut self) -> Result<&mut Runtime, String> {
        let name = self.spec.id.to_string();
        self.runtime
            .as_mut()
            .ok_or_else(|| format!("SESSION_CLOSED {name}"))
    }

    fn activate(&mut self) {
        assert_eq!(
            self.state,
            SessionState::New,
            "session must start exactly once"
        );
        self.state = SessionState::Active;
    }

    fn release(&mut self) -> Option<SessionMountId> {
        if self.state == SessionState::Closed {
            return None;
        }
        self.last_namespace = self
            .runtime
            .as_ref()
            .map(Runtime::current_namespace)
            .unwrap_or_else(|| self.last_namespace.clone());
        if let Some(runtime) = self.runtime.as_mut() {
            runtime.providers.set_file(None);
        }
        let mount = self.filesystem.take().map(|filesystem| filesystem.id);
        self.runtime.take();
        self.authority = SessionAuthorityPolicy::ZERO;
        self.state = SessionState::Closed;
        mount
    }

    pub fn eval(&mut self, source: &str) -> Result<String, String> {
        self.ensure_active()?;
        self.runtime_mut()?.eval_transfer_text(source)
    }

    pub fn current_namespace(&self) -> String {
        self.runtime
            .as_ref()
            .map(Runtime::current_namespace)
            .unwrap_or_else(|| self.last_namespace.clone())
    }

    pub fn authority(&self) -> SessionAuthorityPolicy {
        self.authority
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn install_native_socket_provider(&mut self) {
        self.runtime_mut()
            .expect("closed sessions cannot install providers")
            .install_native_socket_provider();
        self.authority.host_network = true;
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn install_native_process_provider(&mut self) {
        self.runtime_mut()
            .expect("closed sessions cannot install providers")
            .install_native_process_provider();
        self.authority.host_process = true;
    }
}

impl crate::lang::protocol::IContext<&str> for Session {
    type Output = Result<String, String>;

    fn call(&mut self, source: &str) -> Self::Output {
        self.eval(source)
    }
}

impl crate::lang::protocol::IComponent for Session {
    type Metadata = SessionMetadata;

    fn props(&self) -> Self::Metadata {
        SessionStatus {
            name: self.id().clone(),
            namespace: self.current_namespace(),
            state: self.state,
            filesystem: self.filesystem_mount(),
            authority: self.authority,
        }
    }

    fn status(&self) -> Self::Metadata {
        self.props()
    }

    fn started(&self) -> bool {
        self.state == SessionState::Active
    }

    fn stopped(&self) -> bool {
        self.state == SessionState::Closed
    }

    fn start(&mut self) {
        self.activate();
    }

    fn stop(&mut self) {
        self.release();
    }
}

impl<'a> crate::lang::protocol::IApplicable<Session, &'a str> for Session {
    type Output = Result<String, String>;

    fn apply_in(&self, runtime: &mut Session, source: &'a str) -> Self::Output {
        self.ensure_active()?;
        crate::lang::protocol::IContext::call(runtime, source)
    }

    fn apply_default(&mut self) -> &mut Session {
        self
    }

    fn transform_in(&self, _runtime: &Session, source: &'a str) -> &'a str {
        source
    }

    fn transform_out(
        &self,
        _runtime: &Session,
        _source: &'a str,
        value: Self::Output,
    ) -> Self::Output {
        value
    }
}

impl<'a> crate::lang::protocol::IInvokeIn<Session, &'a str> for Session {
    type Output = Result<String, String>;

    fn invoke_in(&self, context: &mut Session, source: &'a str) -> Self::Output {
        self.ensure_active()?;
        crate::lang::protocol::IContext::call(context, source)
    }
}

struct FilesystemMount {
    provider: Rc<dyn core::FileProvider>,
    kind: &'static str,
    key: String,
    attachments: usize,
}

impl Default for SessionKernel {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionKernel {
    pub fn new() -> Self {
        let root_id = SessionId::parse("ROOT").expect("ROOT is a valid session identifier");
        Self {
            sessions: HashMap::from([(
                root_id.to_string(),
                Session::open(
                    SessionSpec::new(root_id, SessionAuthorityPolicy::ZERO),
                    Runtime::new(),
                ),
            )]),
            resources: HashMap::new(),
            mounts: HashMap::new(),
            session_mounts: HashMap::new(),
            next_mount_id: 1,
            test_runner: "code.test".into(),
        }
    }

    pub fn set_test_runner(&mut self, runner: &str) -> Result<(), String> {
        validate_test_runner(runner)?;
        self.test_runner = runner.into();
        for session in self.sessions.values_mut() {
            session.runtime_mut()?.configure_test_runner(runner)?;
        }
        Ok(())
    }

    pub fn create_session(&mut self, id: SessionId) -> Result<(), String> {
        let spec = SessionSpec::new(id, SessionAuthorityPolicy::ZERO);
        if self.sessions.contains_key(spec.id.as_str()) {
            return Err(format!("SESSION_EXISTS {}", spec.id));
        }
        let mut runtime = Runtime::new();
        runtime.configure_test_runner(&self.test_runner)?;
        for (resource, source) in &self.resources {
            runtime.register_resource(resource, source);
        }
        self.sessions
            .insert(spec.id.as_str().into(), Session::open(spec, runtime));
        Ok(())
    }

    pub fn session_names(&self) -> Vec<SessionId> {
        let mut names = self
            .sessions
            .values()
            .map(|session| session.id().clone())
            .collect::<Vec<_>>();
        names.sort();
        names
    }

    pub fn session(&self, id: &SessionId) -> Result<&Session, String> {
        self.sessions
            .get(id.as_str())
            .ok_or_else(|| format!("NO_SESSION {id}"))
    }

    pub fn session_mut(&mut self, id: &SessionId) -> Result<&mut Session, String> {
        self.sessions
            .get_mut(id.as_str())
            .ok_or_else(|| format!("NO_SESSION {id}"))
    }

    pub fn session_namespace(&self, id: &SessionId) -> Result<String, String> {
        self.sessions
            .get(id.as_str())
            .map(Session::current_namespace)
            .ok_or_else(|| format!("NO_SESSION {id}"))
    }

    pub fn eval(&mut self, id: &SessionId, source: &str) -> Result<String, String> {
        self.sessions
            .get_mut(id.as_str())
            .ok_or_else(|| format!("NO_SESSION {id}"))?
            .eval(source)
    }

    pub fn register_resource(&mut self, name: &str, source: &str) {
        self.resources.insert(name.into(), source.into());
        for session in self.sessions.values_mut() {
            session
                .runtime_mut()
                .expect("kernel cannot retain a closed session")
                .register_resource(name, source);
        }
    }

    pub fn create_memory_filesystem(&mut self, root: &str) -> SessionMountId {
        self.create_filesystem(Rc::new(core::MemoryFileProvider::new(root)), "memory", root)
    }

    #[cfg(any(not(target_arch = "wasm32"), target_os = "wasi"))]
    pub fn create_native_filesystem(&mut self, root: &str) -> SessionMountId {
        self.create_filesystem(Rc::new(core::NativeFileProvider::new(root)), "native", root)
    }

    fn create_filesystem(
        &mut self,
        provider: Rc<dyn core::FileProvider>,
        kind: &'static str,
        key: &str,
    ) -> SessionMountId {
        let id = self.next_mount_id;
        self.next_mount_id = self
            .next_mount_id
            .checked_add(1)
            .expect("filesystem mount identifiers exhausted");
        self.mounts.insert(
            id,
            FilesystemMount {
                provider,
                kind,
                key: key.into(),
                attachments: 0,
            },
        );
        SessionMountId::new(id)
    }

    pub fn attach_filesystem(
        &mut self,
        session: &SessionId,
        mount_id: SessionMountId,
    ) -> Result<(), String> {
        if !self.sessions.contains_key(session.as_str()) {
            return Err(format!("NO_SESSION {session}"));
        }
        let provider = self
            .mounts
            .get(&mount_id.get())
            .ok_or_else(|| format!("NO_FILESYSTEM {mount_id}"))?
            .provider
            .clone();
        if self.session_mounts.get(session.as_str()) == Some(&mount_id.get()) {
            return Ok(());
        }
        self.detach_filesystem(session)?;
        self.mounts.get_mut(&mount_id.get()).unwrap().attachments += 1;
        self.session_mounts
            .insert(session.to_string(), mount_id.get());
        let session = self.sessions.get_mut(session.as_str()).unwrap();
        session.runtime_mut()?.providers.set_file(Some(provider.clone()));
        session.filesystem = Some(AttachedFilesystem {
            id: mount_id,
            _provider: provider,
        });
        Ok(())
    }

    pub fn detach_filesystem(&mut self, session: &SessionId) -> Result<(), String> {
        let session = self
            .sessions
            .get_mut(session.as_str())
            .ok_or_else(|| format!("NO_SESSION {session}"))?;
        session.runtime_mut()?.providers.set_file(None);
        session.filesystem.take();
        if let Some(mount_id) = self.session_mounts.remove(session.id().as_str()) {
            if let Some(mount) = self.mounts.get_mut(&mount_id) {
                mount.attachments = mount.attachments.saturating_sub(1);
            }
        }
        Ok(())
    }

    pub fn filesystem(&self, session: &SessionId) -> Option<SessionMountId> {
        self.sessions
            .get(session.as_str())
            .and_then(Session::filesystem_mount)
    }

    pub fn filesystem_info(
        &self,
        mount_id: SessionMountId,
    ) -> Result<(&str, &str, usize), String> {
        self.mounts
            .get(&mount_id.get())
            .map(|mount| (mount.kind, mount.key.as_str(), mount.attachments))
            .ok_or_else(|| format!("NO_FILESYSTEM {mount_id}"))
    }

    pub fn close_filesystem(&mut self, mount_id: SessionMountId) -> Result<(), String> {
        let mount = self
            .mounts
            .get(&mount_id.get())
            .ok_or_else(|| format!("NO_FILESYSTEM {mount_id}"))?;
        if mount.attachments != 0 {
            return Err(format!("FILESYSTEM_ATTACHED {mount_id}"));
        }
        self.mounts.remove(&mount_id.get());
        Ok(())
    }

    pub fn close_session(&mut self, id: &SessionId) -> Result<(), String> {
        if id.as_str() == "ROOT" {
            return Err("ROOT_CANNOT_CLOSE".into());
        }
        if !self.sessions.contains_key(id.as_str()) {
            return Err(format!("NO_SESSION {id}"));
        }
        self.detach_filesystem(id)?;
        if let Some(mut session) = self.sessions.remove(id.as_str()) {
            crate::lang::protocol::IComponent::stop(&mut session);
        }
        Ok(())
    }
}

fn validate_test_runner(runner: &str) -> Result<(), String> {
    if matches!(runner, "code.test" | "native") {
        Ok(())
    } else {
        Err("runtime test runner must be code.test or native".into())
    }
}

/// The root Foundation surface deliberately contains only the iterator core.
/// Native iterator mechanics must enter through the `Iter/*` type alias, so
/// reject legacy unqualified call heads before namespace rewriting canonicalizes
/// an alias to its backing method name.
fn reject_legacy_iterator_calls(form: &Form) -> Result<(), String> {
    const LEGACY: &[&str] = &[
        "iter-has?",
        "iter-finite?",
        "iter-materialize",
        "iter-close",
        "iter-map",
        "iter-filter",
        "iter-take-while",
        "iter-drop-while",
        "iter-mapcat",
        "iter-keep",
        "iter-interpose",
        "iter-interleave",
        "iter-every?",
        "iter-any?",
        "iter-take",
        "iter-drop",
        "iter-zip",
        "iter-cycle",
        "iter-partition-pair",
        "iter-partition-all",
        "iter-partition",
        "iter-range",
        "iter-constantly",
        "iter-repeatedly",
        "iter-iterate",
    ];
    match form {
        Form::List(values) => {
            if let Some(Form::Symbol(name)) = values.first() {
                if LEGACY.contains(&name.as_str()) {
                    return Err(format!("unbound symbol: {name}"));
                }
                if name == "quote" {
                    return Ok(());
                }
            }
            for value in values {
                reject_legacy_iterator_calls(value)?;
            }
        }
        Form::Vector(values) | Form::Set(values) => {
            for value in values {
                reject_legacy_iterator_calls(value)?;
            }
        }
        Form::Map(entries) => {
            for (key, value) in entries {
                reject_legacy_iterator_calls(key)?;
                reject_legacy_iterator_calls(value)?;
            }
        }
        Form::Tagged(_, value) | Form::Metadata(_, value) => reject_legacy_iterator_calls(value)?,
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod authority_tests {
    use super::*;

    fn session_id(name: &str) -> SessionId {
        SessionId::parse(name).unwrap()
    }

    #[test]
    fn named_sessions_start_with_zero_host_authority() {
        let mut kernel = SessionKernel::new();
        let root = session_id("ROOT");
        #[cfg(not(target_arch = "wasm32"))]
        {
            kernel
                .session_mut(&root)
                .unwrap()
                .install_native_socket_provider();
            kernel
                .session_mut(&root)
                .unwrap()
                .install_native_process_provider();
            assert_eq!(
                kernel.session(&root).unwrap().authority().profile(),
                "explicit"
            );
        }

        let child_id = session_id("child");
        kernel.create_session(child_id.clone()).unwrap();
        let child = kernel.session(&child_id).unwrap();
        assert_eq!(child.authority(), SessionAuthorityPolicy::ZERO);
        assert_eq!(
            crate::lang::protocol::IComponent::props(child)
                .authority
                .profile(),
            "zero"
        );

        for capability in ["filesystem", "network/socket", "process"] {
            let error = kernel
                .eval(
                    &child_id,
                    &format!("(deref (Host/capability? \"{capability}\"))"),
                )
                .unwrap_err();
            assert!(error.contains("Host capability provider is unavailable"));
            assert!(error.contains(":host/unavailable"));
        }
    }

    #[test]
    fn session_status_uses_typed_identity_state_and_mount() {
        use crate::lang::protocol::IComponent;

        let mut kernel = SessionKernel::new();
        let typed = session_id("typed");
        kernel.create_session(typed.clone()).unwrap();
        let initial = kernel.session(&typed).unwrap().props();
        assert_eq!(initial.name.as_str(), "typed");
        assert_eq!(initial.state, SessionState::Active);
        assert_eq!(initial.filesystem, None);

        let mount = kernel.create_memory_filesystem("/");
        kernel.attach_filesystem(&typed, mount).unwrap();
        let mounted = kernel.session(&typed).unwrap().props();
        assert_eq!(mounted.filesystem, Some(mount));
        assert_eq!(kernel.session(&typed).unwrap().spec().id, mounted.name);
    }

    #[test]
    fn scoped_filesystem_mount_does_not_change_host_authority_profile() {
        let mut kernel = SessionKernel::new();
        let mounted = session_id("mounted");
        kernel.create_session(mounted.clone()).unwrap();
        let mount = kernel.create_memory_filesystem("/");
        kernel.attach_filesystem(&mounted, mount).unwrap();
        assert_eq!(
            kernel.session(&mounted).unwrap().authority(),
            SessionAuthorityPolicy::ZERO
        );
        assert_eq!(kernel.filesystem(&mounted), Some(mount));
    }

    #[test]
    fn closing_releases_session_owned_runtime_and_filesystem_once() {
        use crate::lang::protocol::IComponent;

        let mut kernel = SessionKernel::new();
        let child = session_id("owned");
        kernel.create_session(child.clone()).unwrap();
        let mount = kernel.create_memory_filesystem("/");
        assert_eq!(Rc::strong_count(&kernel.mounts[&mount.get()].provider), 1);

        kernel.attach_filesystem(&child, mount).unwrap();
        assert_eq!(Rc::strong_count(&kernel.mounts[&mount.get()].provider), 3);

        let mut session = kernel.sessions.remove(child.as_str()).unwrap();
        let released_mount = session.release();
        assert_eq!(released_mount, Some(mount));
        assert_eq!(session.state(), SessionState::Closed);
        assert!(session.runtime.is_none());
        assert!(session.filesystem.is_none());
        assert_eq!(Rc::strong_count(&kernel.mounts[&mount.get()].provider), 1);

        assert_eq!(session.release(), None);
        session.stop();
        assert_eq!(Rc::strong_count(&kernel.mounts[&mount.get()].provider), 1);
    }
}
