package hara.truffle;

import hara.lang.protocol.Constant;
import hara.lang.protocol.IMetadata;
import java.nio.file.Path;
import java.util.Objects;

/** Typed construction and status boundaries shared by Java runtime sessions. */
final class SessionModel {
  private SessionModel() {}

  /** Stable identity for one process-local Hara session. */
  record SessionId(String value) implements Comparable<SessionId> {
    SessionId {
      if (value == null || value.isEmpty() || !value.matches("[A-Za-z0-9_.-]+")) {
        throw new IllegalArgumentException("INVALID_SESSION_NAME");
      }
    }

    static SessionId parse(String value) {
      return new SessionId(value);
    }

    @Override
    public int compareTo(SessionId other) {
      return value.compareTo(other.value);
    }

    @Override
    public String toString() {
      return value;
    }
  }

  /** Identity of one filesystem delegation attached to a session. */
  record SessionMountId(Path root) {
    SessionMountId {
      root = Objects.requireNonNull(root, "root").toAbsolutePath().normalize();
    }

    static SessionMountId from(Path root) {
      return new SessionMountId(root);
    }

    @Override
    public String toString() {
      return root.toString();
    }
  }

  /** Observable lifecycle of one session. */
  enum SessionState {
    NEW("new"),
    ACTIVE("active"),
    CLOSED("closed");

    private final String display;

    SessionState(String display) {
      this.display = display;
    }

    @Override
    public String toString() {
      return display;
    }
  }

  /** Immutable construction contract for one session. */
  record SessionSpec(SessionId id, SessionKernel.SessionAuthorityPolicy authority) {
    SessionSpec {
      Objects.requireNonNull(id, "id");
      Objects.requireNonNull(authority, "authority");
    }

    static SessionSpec zeroAuthority(SessionId id) {
      return new SessionSpec(id, SessionKernel.SessionAuthorityPolicy.ZERO);
    }
  }

  /** Immutable status projection for one session. */
  record SessionStatus(
      SessionId name,
      String namespace,
      SessionState state,
      SessionMountId filesystem,
      SessionKernel.SessionAuthorityPolicy authority)
      implements IMetadata {
    SessionStatus {
      Objects.requireNonNull(name, "name");
      Objects.requireNonNull(state, "state");
      Objects.requireNonNull(authority, "authority");
    }

    @Override
    public Constant.MetaType getMetatype() {
      return Constant.MetaType.MAP;
    }
  }
}
