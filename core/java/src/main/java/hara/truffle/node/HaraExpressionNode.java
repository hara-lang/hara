package hara.truffle.node;

import com.oracle.truffle.api.frame.VirtualFrame;
import com.oracle.truffle.api.instrumentation.GenerateWrapper;
import com.oracle.truffle.api.instrumentation.InstrumentableNode;
import com.oracle.truffle.api.instrumentation.ProbeNode;
import com.oracle.truffle.api.instrumentation.StandardTags;
import com.oracle.truffle.api.instrumentation.Tag;
import com.oracle.truffle.api.nodes.Node;
import com.oracle.truffle.api.source.SourceSection;
import hara.truffle.HaraInstrumentationTags;

/** Base expression node with generated Truffle instrumentation wrappers. */
@GenerateWrapper
public abstract class HaraExpressionNode extends Node implements InstrumentableNode {
  private SourceSection sourceSection;
  private boolean executionRoot;

  public abstract Object execute(VirtualFrame frame);

  public void setHaraSourceSection(SourceSection sourceSection) {
    this.sourceSection = sourceSection;
  }

  void markExecutionRoot(SourceSection fallback) {
    executionRoot = true;
    if (sourceSection == null) sourceSection = fallback;
  }

  @Override
  public boolean isInstrumentable() {
    return sourceSection != null || executionRoot;
  }

  @Override
  public WrapperNode createWrapper(ProbeNode probe) {
    return new HaraExpressionNodeWrapper(this, probe);
  }

  @Override
  public boolean hasTag(Class<? extends Tag> tag) {
    return tag == StandardTags.ExpressionTag.class
        || (executionRoot && tag == HaraInstrumentationTags.ExecutionRootTag.class);
  }

  @Override
  public SourceSection getSourceSection() {
    return sourceSection;
  }
}
