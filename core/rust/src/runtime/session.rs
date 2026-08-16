/// Authority inherited from an embedding host by one in-process evaluator.
///
/// This policy describes host authority only. A filesystem mounted explicitly
/// through [`SessionKernel::attach_filesystem`] is a separately delegated,
/// scoped resource and does not turn a child into a host-authority session.
/// Namespace and runtime separation remain logical isolation, not a security
/// boundary.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SessionAuthorityPolicy {
    pub host_filesystem: bool,
    pub host_network: bool,
    pub host_process: bool,
    pub reflection: bool,
    pub packages: bool,
    pub project: bool,
}

impl SessionAuthorityPolicy {
    pub const ZERO: Self = Self {
        host_filesystem: false,
        host_network: false,
        host_process: false,
        reflection: false,
        packages: false,
        project: false,
    };

    pub const fn profile(self) -> &'static str {
        if !self.host_filesystem
            && !self.host_network
            && !self.host_process
            && !self.reflection
            && !self.packages
            && !self.project
        {
            "zero"
        } else {
            "explicit"
        }
    }
}

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionMetadata {
    pub name: String,
    pub namespace: String,
    pub state: &'static str,
    pub filesystem: Option<u64>,
    pub authority: SessionAuthorityPolicy,
}

/// An isolated, named execution context owned by a [`SessionKernel`].
pub struct Session {
    name: String,
    runtime: Runtime,
    active: bool,
    filesystem: Option<u64>,
    authority: SessionAuthorityPolicy,
}

impl Session {
    fn new(name: &str, runtime: Runtime, authority: SessionAuthorityPolicy) -> Self {
        Self {
            name: name.into(),
            runtime,
            active: true,
            filesystem: None,
            authority,
        }
    }

    fn ensure_active(&self) -> Result<(), String> {
        if self.active {
            Ok(())
        } else {
            Err(format!("SESSION_CLOSED {}", self.name))
        }
    }

    pub fn eval(&mut self, source: &str) -> Result<String, String> {
        self.ensure_active()?;
        self.runtime.eval_transfer_text(source)
    }

    pub fn current_namespace(&self) -> String {
        self.runtime.current_namespace()
    }

    pub fn authority(&self) -> SessionAuthorityPolicy {
        self.authority
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn install_native_socket_provider(&mut self) {
        self.runtime.install_native_socket_provider();
        self.authority.host_network = true;
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn install_native_process_provider(&mut self) {
        self.runtime.install_native_process_provider();
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
        SessionMetadata {
            name: self.name.clone(),
            namespace: self.current_namespace(),
            state: if self.active { "idle" } else { "closed" },
            filesystem: self.filesystem,
            authority: self.authority,
        }
    }

    fn status(&self) -> Self::Metadata {
        self.props()
    }

    fn started(&self) -> bool {
        self.active
    }

    fn stopped(&self) -> bool {
        !self.active
    }

    fn start(&mut self) {
        assert!(self.active, "cannot restart closed session {}", self.name);
    }

    fn stop(&mut self) {
        self.active = false;
        self.filesystem = None;
        self.authority = SessionAuthorityPolicy::ZERO;
        self.runtime.providers.set_file(None);
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
        Self {
            sessions: HashMap::from([(
                "ROOT".into(),
                Session::new("ROOT", Runtime::new(), SessionAuthorityPolicy::ZERO),
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
            session.runtime.configure_test_runner(runner)?;
        }
        Ok(())
    }

    pub fn create_session(&mut self, name: &str) -> Result<(), String> {
        validate_session_name(name)?;
        if self.sessions.contains_key(name) {
            return Err(format!("SESSION_EXISTS {name}"));
        }
        let mut runtime = Runtime::new();
        runtime.configure_test_runner(&self.test_runner)?;
        for (resource, source) in &self.resources {
            runtime.register_resource(resource, source);
        }
        self.sessions.insert(
            name.into(),
            Session::new(name, runtime, SessionAuthorityPolicy::ZERO),
        );
        Ok(())
    }

    pub fn session_names(&self) -> Vec<String> {
        let mut names = self.sessions.keys().cloned().collect::<Vec<_>>();
        names.sort();
        names
    }

    pub fn session(&self, name: &str) -> Result<&Session, String> {
        self.sessions
            .get(name)
            .ok_or_else(|| format!("NO_SESSION {name}"))
    }

    pub fn session_mut(&mut self, name: &str) -> Result<&mut Session, String> {
        self.sessions
            .get_mut(name)
            .ok_or_else(|| format!("NO_SESSION {name}"))
    }

    pub fn session_namespace(&self, session: &str) -> Result<String, String> {
        self.sessions
            .get(session)
            .map(Session::current_namespace)
            .ok_or_else(|| format!("NO_SESSION {session}"))
    }

    pub fn eval(&mut self, session: &str, source: &str) -> Result<String, String> {
        self.sessions
            .get_mut(session)
            .ok_or_else(|| format!("NO_SESSION {session}"))?
            .eval(source)
    }

    pub fn register_resource(&mut self, name: &str, source: &str) {
        self.resources.insert(name.into(), source.into());
        for session in self.sessions.values_mut() {
            session.runtime.register_resource(name, source);
        }
    }

    pub fn create_memory_filesystem(&mut self, root: &str) -> u64 {
        self.create_filesystem(Rc::new(core::MemoryFileProvider::new(root)), "memory", root)
    }

    #[cfg(any(not(target_arch = "wasm32"), target_os = "wasi"))]
    pub fn create_native_filesystem(&mut self, root: &str) -> u64 {
        self.create_filesystem(Rc::new(core::NativeFileProvider::new(root)), "native", root)
    }

    fn create_filesystem(
        &mut self,
        provider: Rc<dyn core::FileProvider>,
        kind: &'static str,
        key: &str,
    ) -> u64 {
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
        id
    }

    pub fn attach_filesystem(&mut self, session: &str, mount_id: u64) -> Result<(), String> {
        if !self.sessions.contains_key(session) {
            return Err(format!("NO_SESSION {session}"));
        }
        let provider = self
            .mounts
            .get(&mount_id)
            .ok_or_else(|| format!("NO_FILESYSTEM {mount_id}"))?
            .provider
            .clone();
        if self.session_mounts.get(session) == Some(&mount_id) {
            return Ok(());
        }
        self.detach_filesystem(session)?;
        self.mounts.get_mut(&mount_id).unwrap().attachments += 1;
        self.session_mounts.insert(session.into(), mount_id);
        let session = self.sessions.get_mut(session).unwrap();
        session.runtime.providers.set_file(Some(provider));
        session.filesystem = Some(mount_id);
        Ok(())
    }

    pub fn detach_filesystem(&mut self, session: &str) -> Result<(), String> {
        let runtime = self
            .sessions
            .get_mut(session)
            .ok_or_else(|| format!("NO_SESSION {session}"))?;
        runtime.runtime.providers.set_file(None);
        runtime.filesystem = None;
        if let Some(mount_id) = self.session_mounts.remove(session) {
            if let Some(mount) = self.mounts.get_mut(&mount_id) {
                mount.attachments = mount.attachments.saturating_sub(1);
            }
        }
        Ok(())
    }

    pub fn filesystem(&self, session: &str) -> Option<u64> {
        self.session_mounts.get(session).copied()
    }

    pub fn filesystem_info(&self, mount_id: u64) -> Result<(&str, &str, usize), String> {
        self.mounts
            .get(&mount_id)
            .map(|mount| (mount.kind, mount.key.as_str(), mount.attachments))
            .ok_or_else(|| format!("NO_FILESYSTEM {mount_id}"))
    }

    pub fn close_filesystem(&mut self, mount_id: u64) -> Result<(), String> {
        let mount = self
            .mounts
            .get(&mount_id)
            .ok_or_else(|| format!("NO_FILESYSTEM {mount_id}"))?;
        if mount.attachments != 0 {
            return Err(format!("FILESYSTEM_ATTACHED {mount_id}"));
        }
        self.mounts.remove(&mount_id);
        Ok(())
    }

    pub fn close_session(&mut self, name: &str) -> Result<(), String> {
        validate_session_name(name)?;
        if name == "ROOT" {
            return Err("ROOT_CANNOT_CLOSE".into());
        }
        if !self.sessions.contains_key(name) {
            return Err(format!("NO_SESSION {name}"));
        }
        self.detach_filesystem(name)?;
        if let Some(mut session) = self.sessions.remove(name) {
            crate::lang::protocol::IComponent::stop(&mut session);
        }
        Ok(())
    }
}

fn validate_session_name(name: &str) -> Result<(), String> {
    if name.is_empty()
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'-'))
    {
        return Err("INVALID_SESSION_NAME".into());
    }
    Ok(())
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

    #[test]
    fn named_sessions_start_with_zero_host_authority() {
        let mut kernel = SessionKernel::new();
        #[cfg(not(target_arch = "wasm32"))]
        {
            kernel
                .session_mut("ROOT")
                .unwrap()
                .install_native_socket_provider();
            kernel
                .session_mut("ROOT")
                .unwrap()
                .install_native_process_provider();
            assert_eq!(
                kernel.session("ROOT").unwrap().authority().profile(),
                "explicit"
            );
        }

        kernel.create_session("child").unwrap();
        let child = kernel.session("child").unwrap();
        assert_eq!(child.authority(), SessionAuthorityPolicy::ZERO);
        assert_eq!(child.props().authority.profile(), "zero");

        assert_eq!(
            kernel
                .eval("child", "(deref (Host/capability? \"filesystem\"))")
                .unwrap(),
            "false"
        );
        assert_eq!(
            kernel
                .eval("child", "(deref (Host/capability? \"network/socket\"))")
                .unwrap(),
            "false"
        );
        assert_eq!(
            kernel
                .eval("child", "(deref (Host/capability? \"process\"))")
                .unwrap(),
            "false"
        );
    }

    #[test]
    fn scoped_filesystem_mount_does_not_change_host_authority_profile() {
        let mut kernel = SessionKernel::new();
        kernel.create_session("mounted").unwrap();
        let mount = kernel.create_memory_filesystem("/");
        kernel.attach_filesystem("mounted", mount).unwrap();
        assert_eq!(
            kernel.session("mounted").unwrap().authority(),
            SessionAuthorityPolicy::ZERO
        );
        assert_eq!(kernel.filesystem("mounted"), Some(mount));
    }
}
