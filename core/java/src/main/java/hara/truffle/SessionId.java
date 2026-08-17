package hara.truffle;

import java.util.Objects;

/** Stable identity for one process-local Hara Session. */
final class SessionId implements Comparable<SessionId> {
  private final String value;

  private SessionId(String value) {
    this.value = value;
  }

  static SessionId parse(String value) {
    if (value == null || value.isEmpty() || !value.matches("[A-Za-z0-9_.-]+")) {
      throw new IllegalArgumentException("INVALID_SESSION_NAME");
    }
    return new SessionId(value);
  }

  String value() {
    return value;
  }

  @Override
  public int compareTo(SessionId other) {
    return value.compareTo(other.value);
  }

  @Override
  public boolean equals(Object other) {
    return other instanceof SessionId sessionId && value.equals(sessionId.value);
  }

  @Override
  public int hashCode() {
    return Objects.hash(value);
  }

  @Override
  public String toString() {
    return value;
  }
}
