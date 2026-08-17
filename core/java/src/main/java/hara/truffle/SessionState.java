package hara.truffle;

/** Observable lifecycle state of one Session. */
enum SessionState {
  IDLE("idle"),
  BUSY("busy"),
  CLOSED("closed");

  private final String value;

  SessionState(String value) {
    this.value = value;
  }

  String value() {
    return value;
  }

  @Override
  public String toString() {
    return value;
  }
}
