#!/usr/bin/env python3
from pathlib import Path

kernel = Path("core/java/src/main/java/hara/truffle/SessionKernel.java")
text = kernel.read_text()


def replace_once(old: str, new: str) -> None:
    global text
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"expected one replacement, found {count}: {old[:100]!r}")
    text = text.replace(old, new, 1)


text = text.replace("import hara.lang.protocol.Constant;\n", "")
text = text.replace("import hara.lang.protocol.IMetadata;\n", "")
text = text.replace("import java.util.Set;\n", "import java.util.Set;\nimport java.util.TreeSet;\n")

start = text.index("  /**\n   * Host authority applied when a context is created.")
end = text.index("  private final boolean allowFile;", start)
text = text[:start] + text[end:]

replace_once(
    '''  private final boolean allowFile;
  private final ConcurrentHashMap<String, Session> sessions = new ConcurrentHashMap<>();''',
    '''  private final boolean allowFile;
  private final ConcurrentHashMap<SessionId, Session> sessions = new ConcurrentHashMap<>();''',
)

replace_once(
    '''    this.allowFile = allowFile;
    SessionAuthorityPolicy rootAuthority =
        SessionAuthorityPolicy.root(allowFile, allowNetwork, allowProcess, project);
    sessions.put("ROOT", new Session("ROOT", rootAuthority, project));''',
    '''    this.allowFile = allowFile;
    SessionSpec rootSpec =
        SessionSpec.root("ROOT", allowFile, allowNetwork, allowProcess, project);
    sessions.put(rootSpec.id, new Session(rootSpec, project));''',
)

replace_once(
    '''  Session require(String name) {
    Session session = sessions.get(name);
    if (session == null) throw new IllegalArgumentException("NO_SESSION " + name);
    return session;
  }

  synchronized Session create(String value) {
    String name = normalizeName(value);
    if (sessions.containsKey(name)) throw new IllegalArgumentException("SESSION_EXISTS " + name);
    Session session = new Session(name, SessionAuthorityPolicy.ZERO, null);
    sessions.put(name, session);
    return session;
  }''',
    '''  Session require(String name) {
    SessionId id = SessionId.parse(name);
    Session session = sessions.get(id);
    if (session == null) throw new IllegalArgumentException("NO_SESSION " + id);
    return session;
  }

  synchronized Session create(String value) {
    SessionSpec spec = SessionSpec.zeroAuthority(value);
    if (sessions.containsKey(spec.id)) {
      throw new IllegalArgumentException("SESSION_EXISTS " + spec.id);
    }
    Session session = new Session(spec, null);
    sessions.put(spec.id, session);
    return session;
  }''',
)

replace_once(
    '''  synchronized void closeSession(String value) {
    String name = normalizeName(value);
    if ("ROOT".equals(name)) throw new IllegalArgumentException("ROOT_CANNOT_CLOSE");
    Session removed = sessions.remove(name);
    if (removed == null) throw new IllegalArgumentException("NO_SESSION " + name);
    removed.close();
  }

  Set<String> sessionNames() {
    return Collections.unmodifiableSet(sessions.keySet());
  }''',
    '''  synchronized void closeSession(String value) {
    SessionId id = SessionId.parse(value);
    if ("ROOT".equals(id.value())) throw new IllegalArgumentException("ROOT_CANNOT_CLOSE");
    Session removed = sessions.remove(id);
    if (removed == null) throw new IllegalArgumentException("NO_SESSION " + id);
    removed.close();
  }

  Set<String> sessionNames() {
    TreeSet<String> names = new TreeSet<>();
    for (SessionId id : sessions.keySet()) names.add(id.value());
    return Collections.unmodifiableSet(names);
  }''',
)

replace_once(
    '''  static String normalizeName(String value) {
    if (value == null || value.isEmpty() || !value.matches("[A-Za-z0-9_.-]+"))
      throw new IllegalArgumentException("INVALID_SESSION_NAME");
    return value;
  }''',
    '''  static String normalizeName(String value) {
    return SessionId.parse(value).value();
  }''',
)

replace_once(
    '''    private final String name;
    private final SessionAuthorityPolicy authority;
    private final HaraProject project;''',
    '''    private final SessionSpec spec;
    private final HaraProject project;''',
)

start = text.index("    static final class SessionMetadata implements IMetadata")
end = text.index("    private Session(", start)
text = text[:start] + text[end:]

replace_once(
    '''    private Session(String name, SessionAuthorityPolicy authority, HaraProject project) {
      this.name = name;
      this.authority = authority;
      this.project = project;
      context = createContext(null);
    }''',
    '''    private Session(SessionSpec spec, HaraProject project) {
      this.spec = spec;
      this.project = project;
      context = createContext(null);
    }''',
)

text = text.replace("authority.hostNetwork", "spec.authority.hostNetwork")
text = text.replace("authority.hostFilesystem", "spec.authority.hostFilesystem")
text = text.replace("authority.hostProcess", "spec.authority.hostProcess")
text = text.replace("authority.project", "spec.authority.project")
text = text.replace("authority.reflection", "spec.authority.reflection")

replace_once(
    '''    private void requireActive() {
      if (!active.get()) throw new IllegalStateException("SESSION_CLOSED " + name);
    }''',
    '''    private void requireActive() {
      if (!active.get()) throw new IllegalStateException("SESSION_CLOSED " + name());
    }''',
)

text = text.replace('"SESSION_BUSY " + name', '"SESSION_BUSY " + name()')

replace_once(
    '''    String name() {
      return name;
    }

    SessionAuthorityPolicy authority() {
      return authority;
    }''',
    '''    SessionSpec spec() {
      return spec;
    }

    String name() {
      return spec.id.value();
    }

    SessionState state() {
      if (!active.get()) return SessionState.CLOSED;
      return activeEvaluations.get() == 0 ? SessionState.IDLE : SessionState.BUSY;
    }

    SessionAuthorityPolicy authority() {
      return spec.authority;
    }''',
)

text = text.replace('"SESSION_CALL_EXPECTS_SOURCE " + name', '"SESSION_CALL_EXPECTS_SOURCE " + name()')
text = text.replace('"SESSION_APPLY_EXPECTS_CONTEXT " + name', '"SESSION_APPLY_EXPECTS_CONTEXT " + name()')

replace_once(
    '''    private SessionMetadata metadata() {
      boolean running = active.get();
      String filesystem =
          filesystemRoot == null
              ? (spec.authority.hostFilesystem ? "HOST" : null)
              : filesystemRoot.toString();
      return new SessionMetadata(
          name,
          running ? currentNamespace() : null,
          running ? (activeEvaluations.get() == 0 ? "idle" : "busy") : "closed",
          filesystem,
          authority.profile());
    }''',
    '''    private SessionStatus metadata() {
      boolean running = active.get();
      return new SessionStatus(
          spec.id,
          running ? currentNamespace() : null,
          state(),
          filesystemRoot,
          spec.authority);
    }''',
)

text = text.replace('"cannot restart closed session " + name', '"cannot restart closed session " + name()')

# info() retains its wire-compatible text shape while deriving it from typed state.
replace_once(
    '''      return List.of(
          "NAME", name,
          "STATE", "RUNNING",
          "FILESYSTEM", filesystem,
          "AUTHORITY", authority.profile());''',
    '''      return List.of(
          "NAME", name(),
          "STATE", state().value().toUpperCase(),
          "FILESYSTEM", filesystem,
          "AUTHORITY", spec.authority.profile());''',
)

kernel.write_text(text)

# Update tests to use the extracted types and typed status projection.
test = Path("core/java/src/test/java/hara/truffle/SessionKernelTest.java")
source = test.read_text()
source = source.replace(
    "SessionKernel.SessionAuthorityPolicy policy = child.authority();",
    "SessionAuthorityPolicy policy = child.authority();",
)
source = source.replace(
    "((SessionKernel.Session.SessionMetadata) child.getStatus()).authority",
    "((SessionStatus) child.getStatus()).authority.profile()",
)
source = source.replace(
    "((SessionKernel.Session.SessionMetadata) alpha.getProps()).namespace",
    "((SessionStatus) alpha.getProps()).namespace",
)
source = source.replace(
    "((SessionKernel.Session.SessionMetadata) alpha.getProps()).authority",
    "((SessionStatus) alpha.getProps()).authority.profile()",
)

marker = '''  @Test
  public void sessionsConformToContextComponentAndApplicativeProtocols() {'''
addition = '''  @Test
  public void sessionTypesSeparateIdentitySpecStateAndMount() throws Exception {
    Path root = Files.createTempDirectory("hara-session-type-model");
    try (SessionKernel kernel = new SessionKernel(true, false)) {
      SessionKernel.Session session = kernel.create("typed.session");
      SessionStatus initial = (SessionStatus) session.getStatus();
      assertEquals("typed.session", initial.id.value());
      assertEquals(SessionState.IDLE, initial.state);
      assertEquals(SessionAuthorityPolicy.ZERO, initial.authority);
      assertEquals(initial.id, session.spec().id);
      assertEquals(null, initial.filesystem);

      kernel.attachFilesystem("typed.session", root);
      SessionStatus mounted = (SessionStatus) session.getStatus();
      assertEquals(root.toAbsolutePath().normalize(), mounted.filesystem);
      assertEquals(SessionAuthorityPolicy.ZERO, mounted.authority);
    } finally {
      Files.deleteIfExists(root);
    }
  }

'''
if source.count(marker) != 1:
    raise SystemExit("expected one Java typed Session test insertion point")
source = source.replace(marker, addition + marker, 1)
test.write_text(source)
