package hara.truffle;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;

import hara.lang.protocol.IClosed;
import hara.lang.protocol.IComponent;
import hara.lang.protocol.IWork;
import hara.lang.protocol.IWorkHost;
import hara.lang.protocol.IWorkRef;
import hara.lang.protocol.IWorkRun;
import java.util.Map;
import org.graalvm.polyglot.Context;
import org.junit.Test;

public class HaraWorkProtocolTest {
  @Test
  public void nativeInterfacesPreserveTheWorkLifecycleHierarchy() {
    assertTrue(IComponent.class.isAssignableFrom(IWorkHost.class));
    assertTrue(IWorkRef.class.isAssignableFrom(IWorkRun.class));
    assertTrue(IClosed.class.isAssignableFrom(IWorkRun.class));
  }

  @Test
  public void adaptsJavaWorkAndReferenceValues() {
    HaraProtocol work = new HaraProtocol("IWork", Map.of("work-spec", 1));
    WorkProtocolLibraryProvider.installWork(work);
    HaraProtocol reference = new HaraProtocol("IWorkRef", Map.of("work-id", 1));
    WorkProtocolLibraryProvider.installWorkRef(reference);

    IWork workValue = () -> Map.of("op", "pure");
    IWorkRef referenceValue = () -> "run-1";

    assertEquals(Map.of("op", "pure"), work.invoke("work-spec", workValue, new Object[0]));
    assertEquals("run-1", reference.invoke("work-id", referenceValue, new Object[0]));
  }

  @Test
  public void guestTypesExtendNativeWorkProtocolsAndParents() {
    try (Context context = Context.newBuilder(HaraLanguage.ID).build()) {
      assertEquals(
          "[{:op :pure} \"run-1\" [:work :input {:priority :high}] \"run-1\" :completed true true]",
          context
              .eval(
                  HaraLanguage.ID,
                  "(do "
                      + "(defstruct NativeWorkFixture [spec]) "
                      + "(extend-type NativeWorkFixture IWork "
                      + "  (work-spec [work] (:spec work))) "
                      + "(defstruct NativeHostFixture [value]) "
                      + "(extend-type NativeHostFixture IComponent "
                      + "  (props [host] {}) "
                      + "  (status [host] :started) "
                      + "  (started? [host] true) "
                      + "  (stopped? [host] false) "
                      + "  (start [host] host) "
                      + "  (stop [host] host) "
                      + "  (kill [host] host) "
                      + "  (remote? [host] false)) "
                      + "(extend-type NativeHostFixture IWorkHost "
                      + "  (work-submit [host work input options] [work input options]) "
                      + "  (work-resolve [host reference] reference)) "
                      + "(defstruct NativeRunFixture [id]) "
                      + "(extend-type NativeRunFixture IWorkRef "
                      + "  (work-id [run] (:id run))) "
                      + "(extend-type NativeRunFixture IClosed "
                      + "  (closed? [run] true)) "
                      + "(extend-type NativeRunFixture IWorkRun "
                      + "  (work-status [run] :completed) "
                      + "  (work-result [run] nil) "
                      + "  (work-events [run options] nil) "
                      + "  (work-cancel [run reason] nil)) "
                      + "(let [work (NativeWorkFixture {:op :pure}) "
                      + "      host (NativeHostFixture nil) "
                      + "      run (NativeRunFixture \"run-1\")] "
                      + "  [(IWork/work-spec work) "
                      + "   (IWorkRef/work-id run) "
                      + "   (IWorkHost/work-submit host :work :input {:priority :high}) "
                      + "   (IWorkHost/work-resolve host \"run-1\") "
                      + "   (IWorkRun/work-status run) "
                      + "   (satisfies? IWorkHost host) "
                      + "   (satisfies? IWorkRun run)]))")
              .toString());
    }
  }
}
