package hara.truffle;

import com.oracle.truffle.api.instrumentation.Tag;

/** Hara-specific Truffle tags that identify portable instrumentation boundaries. */
public final class HaraInstrumentationTags {
  private HaraInstrumentationTags() {}

  /** Marks the expression that completes one top-level Session evaluation. */
  public static final class ExecutionRootTag extends Tag {
    private ExecutionRootTag() {}
  }
}
