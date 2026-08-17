#!/usr/bin/env python3
from pathlib import Path

path = Path("core/rust/src/runtime/session.rs")
text = path.read_text()


def replace_once(old: str, new: str) -> None:
    global text
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"expected one replacement, found {count}: {old[:100]!r}")
    text = text.replace(old, new, 1)


# The policy and status types now live in runtime/session_model.rs.
start = text.index("/// Authority inherited from an embedding host")
end = text.index("/// A process-local kernel", start)
text = text[:start] + text[end:]

start = text.index("#[derive(Clone, Debug, PartialEq, Eq)]\npub struct SessionMetadata")
end = text.index("/// An isolated, named execution context", start)
text = text[:start] + text[end:]

replace_once(
    '''pub struct Session {
    name: String,
    runtime: Runtime,
    active: bool,
    filesystem: Option<u64>,
    authority: SessionAuthorityPolicy,
}''',
    '''pub struct Session {
    spec: SessionSpec,
    runtime: Runtime,
    state: SessionState,
    filesystem: Option<SessionMountId>,
    authority: SessionAuthorityPolicy,
}''',
)

start = text.index("impl Session {\n")
end = text.index("\nimpl crate::lang::protocol::IContext<&str> for Session", start)
text = (
    text[:start]
    + '''impl Session {
    fn new(name: &str, runtime: Runtime) -> Self {
        let spec = SessionSpec::zero_authority(name)
            .expect("Session::new requires a validated session name");
        Self::open(spec, runtime)
    }

    fn open(spec: SessionSpec, runtime: Runtime) -> Self {
        let authority = spec.authority;
        Self {
            spec,
            runtime,
            state: SessionState::Idle,
            filesystem: None,
            authority,
        }
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
        self.filesystem
    }

    fn ensure_active(&self) -> Result<(), String> {
        if self.state != SessionState::Closed {
            Ok(())
        } else {
            Err(format!("SESSION_CLOSED {}", self.name()))
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
'''
    + text[end:]
)

start = text.index("impl crate::lang::protocol::IComponent for Session {\n")
end = text.index("\nimpl<'a> crate::lang::protocol::IApplicable", start)
text = (
    text[:start]
    + '''impl crate::lang::protocol::IComponent for Session {
    type Metadata = SessionMetadata;

    fn props(&self) -> Self::Metadata {
        SessionStatus {
            name: self.id().clone(),
            namespace: self.current_namespace(),
            state: self.state,
            filesystem: self.filesystem,
            authority: self.authority,
        }
    }

    fn status(&self) -> Self::Metadata {
        self.props()
    }

    fn started(&self) -> bool {
        self.state != SessionState::Closed
    }

    fn stopped(&self) -> bool {
        self.state == SessionState::Closed
    }

    fn start(&mut self) {
        assert!(
            self.state != SessionState::Closed,
            "cannot restart closed session {}",
            self.name()
        );
    }

    fn stop(&mut self) {
        self.state = SessionState::Closed;
        self.filesystem = None;
        self.authority = SessionAuthorityPolicy::ZERO;
        self.runtime.providers.set_file(None);
    }
}
'''
    + text[end:]
)

replace_once(
    '''    pub fn create_session(&mut self, name: &str) -> Result<(), String> {
        validate_session_name(name)?;
        if self.sessions.contains_key(name) {
            return Err(format!("SESSION_EXISTS {name}"));
        }
        let mut runtime = Runtime::new();
        runtime.configure_test_runner(&self.test_runner)?;
        for (resource, source) in &self.resources {
            runtime.register_resource(resource, source);
        }
        self.sessions.insert(name.into(), Session::new(name, runtime));
        Ok(())
    }''',
    '''    pub fn create_session(&mut self, name: &str) -> Result<(), String> {
        let spec = SessionSpec::zero_authority(name)?;
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
    }''',
)

replace_once(
    '''        let session = self.sessions.get_mut(session).unwrap();
        session.runtime.providers.set_file(Some(provider));
        session.filesystem = Some(mount_id);''',
    '''        let session = self.sessions.get_mut(session).unwrap();
        session.runtime.providers.set_file(Some(provider));
        session.filesystem = Some(SessionMountId::new(mount_id));''',
)

replace_once(
    '''    pub fn filesystem(&self, session: &str) -> Option<u64> {
        self.session_mounts.get(session).copied()
    }''',
    '''    pub fn filesystem(&self, session: &str) -> Option<u64> {
        self.sessions
            .get(session)
            .and_then(Session::filesystem_mount)
            .map(SessionMountId::get)
    }''',
)

replace_once(
    '''    pub fn close_session(&mut self, name: &str) -> Result<(), String> {
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
''',
    '''    pub fn close_session(&mut self, name: &str) -> Result<(), String> {
        let id = SessionId::parse(name)?;
        if id.as_str() == "ROOT" {
            return Err("ROOT_CANNOT_CLOSE".into());
        }
        if !self.sessions.contains_key(id.as_str()) {
            return Err(format!("NO_SESSION {id}"));
        }
        self.detach_filesystem(id.as_str())?;
        if let Some(mut session) = self.sessions.remove(id.as_str()) {
            crate::lang::protocol::IComponent::stop(&mut session);
        }
        Ok(())
    }
}
''',
)

insert = '''
    #[test]
    fn session_status_uses_typed_identity_state_and_mount() {
        use crate::lang::protocol::IComponent;

        let mut kernel = SessionKernel::new();
        kernel.create_session("typed").unwrap();
        let initial = kernel.session("typed").unwrap().props();
        assert_eq!(initial.name.as_str(), "typed");
        assert_eq!(initial.state, SessionState::Idle);
        assert_eq!(initial.filesystem, None);

        let mount = kernel.create_memory_filesystem("/");
        kernel.attach_filesystem("typed", mount).unwrap();
        let mounted = kernel.session("typed").unwrap().props();
        assert_eq!(mounted.filesystem, Some(SessionMountId::new(mount)));
        assert_eq!(kernel.session("typed").unwrap().spec().id, mounted.name);
    }
'''
marker = "\n    #[test]\n    fn scoped_filesystem_mount_does_not_change_host_authority_profile()"
if text.count(marker) != 1:
    raise SystemExit("expected one typed status test insertion point")
text = text.replace(marker, insert + marker, 1)

path.write_text(text)
