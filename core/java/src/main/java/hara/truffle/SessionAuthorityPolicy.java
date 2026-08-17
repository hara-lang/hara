package hara.truffle;

/**
 * Direct host authority applied when one Session Runtime is created.
 *
 * <p>A filesystem mounted explicitly on a Session is a separate scoped delegation and does not
 * turn the Session into a host-filesystem Session. In-process Runtime and namespace separation is
 * logical isolation, not a security boundary.
 */
final class SessionAuthorityPolicy {
  static final SessionAuthorityPolicy ZERO =
      new SessionAuthorityPolicy(false, false, false, false, false, false);

  final boolean hostFilesystem;
  final boolean hostNetwork;
  final boolean hostProcess;
  final boolean reflection;
  final boolean packages;
  final boolean project;

  SessionAuthorityPolicy(
      boolean hostFilesystem,
      boolean hostNetwork,
      boolean hostProcess,
      boolean reflection,
      boolean packages,
      boolean project) {
    this.hostFilesystem = hostFilesystem;
    this.hostNetwork = hostNetwork;
    this.hostProcess = hostProcess;
    this.reflection = reflection;
    this.packages = packages;
    this.project = project;
  }

  static SessionAuthorityPolicy root(
      boolean allowFile, boolean allowNetwork, boolean allowProcess, HaraProject project) {
    return new SessionAuthorityPolicy(
        allowFile,
        allowNetwork,
        allowProcess,
        project != null && project.hasCapability("jvm/reflection"),
        project != null,
        project != null);
  }

  String profile() {
    return hostFilesystem || hostNetwork || hostProcess || reflection || packages || project
        ? "explicit"
        : "zero";
  }
}
