package hara.truffle;

/** Immutable construction contract for one ordinary in-process Session. */
final class SessionSpec {
  final SessionId id;
  final SessionAuthorityPolicy authority;

  SessionSpec(SessionId id, SessionAuthorityPolicy authority) {
    this.id = id;
    this.authority = authority;
  }

  static SessionSpec zeroAuthority(String name) {
    return new SessionSpec(SessionId.parse(name), SessionAuthorityPolicy.ZERO);
  }

  static SessionSpec root(
      String name,
      boolean allowFile,
      boolean allowNetwork,
      boolean allowProcess,
      HaraProject project) {
    return new SessionSpec(
        SessionId.parse(name),
        SessionAuthorityPolicy.root(allowFile, allowNetwork, allowProcess, project));
  }
}
