package hara.truffle;

import hara.lang.protocol.Constant;
import hara.lang.protocol.IMetadata;
import java.nio.file.Path;

/** Immutable status projection for one Session. */
final class SessionStatus implements IMetadata {
  final SessionId id;
  final String namespace;
  final SessionState state;
  final Path filesystem;
  final SessionAuthorityPolicy authority;

  SessionStatus(
      SessionId id,
      String namespace,
      SessionState state,
      Path filesystem,
      SessionAuthorityPolicy authority) {
    this.id = id;
    this.namespace = namespace;
    this.state = state;
    this.filesystem = filesystem;
    this.authority = authority;
  }

  String name() {
    return id.value();
  }

  @Override
  public Constant.MetaType getMetatype() {
    return Constant.MetaType.MAP;
  }
}
