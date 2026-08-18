package hara.truffle;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;

import hara.lang.protocol.IClosed;
import hara.lang.protocol.IComponent;
import hara.lang.protocol.IWork;
import hara.lang.protocol.IWorkExecutor;
import hara.lang.protocol.IWorkHost;
import hara.lang.protocol.IWorkRef;
import hara.lang.protocol.IWorkRun;
import hara.lang.protocol.IWorkStore;
import java.util.Map;
import org.graalvm.polyglot.Context;
import org.junit.Test;

public class HaraWorkProtocolTest {
  @Test
  public void nativeInterfacesPreserveTheWorkLifecycleHierarchy() {
    assertTrue(IComponent.class.isAssignableFrom(IWorkHost.class));
    assertTrue(IWorkRef.class.isAssignableFrom(IWorkRun.class));
    assertTrue(IClosed.class.isAssignableFrom(IWorkRun.class));
    assertFalse(IComponent.class.isAssignableFrom(IWorkExecutor.class));
    assertFalse(IComponent.class.isAssignableFrom(IWorkStore.class));
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
  public void adaptsJavaExecutorAndStoreValues() {
    HaraProtocol executor =
        new HaraProtocol("IWorkExecutor", Map.of("work-execute", 2));
    WorkProtocolLibraryProvider.installWorkExecutor(executor);
    HaraProtocol store =
        new HaraProtocol(
            "IWorkStore", Map.of("work-query", 2, "work-transact", 2));
    WorkProtocolLibraryProvider.installWorkStore(store);

    IWorkExecutor executorValue = request -> Map.of("executed", request);
    IWorkStore storeValue =
        new IWorkStore() {
          @Override
          public Object workQuery(Object query) {
            return Map.of("query", query);
          }

          @Override
          public Object workTransact(Object transition) {
            return Map.of("transact", transition);
          }
        };

    Map<String, String> request = Map.of("leaf", "compile");
    Map<String, String> query = Map.of("query", "run");
    Map<String, String> transition = Map.of("run", "run-1");

    assertEquals(
        Map.of("executed", request),
        executor.invoke("work-execute", executorValue, new Object[] {request}));
    assertEquals(
        Map.of("query", query),
        store.invoke("work-query", storeValue, new Object[] {query}));
    assertEquals(
        Map.of("transact", transition),
        store.invoke("work-transact", storeValue, new Object[] {transition}));
  }

  @Test
  public void guestTypesExtendNativeWorkProtocolsAndParents() {
    try (Context context = Context.newBuilder(HaraLanguage.ID).build()) {
      assertEquals(
          "[{:op :pure} \"run-1\" [:work :input {:priority :high}] \"run-1\" :completed true true [:executed {:leaf :compile}] [:query {:query/type :run}] [:transact {:transition/run-id \"run-1\"}] true true]",
          context
              .eval(
                  HaraLanguage.ID,
                  "(do "
                      + "(defstruct NativeWorkFixture [spec]) "
                      + "(extend-type NativeWorkFixture IWork "
                      + "  (work-spec [work] (:spec work))) "
                      + "(defstruct NativeExecutorFixture [value]) "
                      + "(extend-type NativeExecutorFixture IWorkExecutor "
                      + "  (work-execute [executor request] [:executed request])) "
                      + "(defstruct NativeStoreFixture [value]) "
                      + "(extend-type NativeStoreFixture IWorkStore "
                      + "  (work-query [store query] [:query query]) "
                      + "  (work-transact [store transition] [:transact transition])) "
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
                      + "      executor (NativeExecutorFixture nil) "
                      + "      store (NativeStoreFixture nil) "
                      + "      host (NativeHostFixture nil) "
                      + "      run (NativeRunFixture \"run-1\")] "
                      + "  [(IWork/work-spec work) "
                      + "   (IWorkRef/work-id run) "
                      + "   (IWorkHost/work-submit host :work :input {:priority :high}) "
                      + "   (IWorkHost/work-resolve host \"run-1\") "
                      + "   (IWorkRun/work-status run) "
                      + "   (satisfies? IWorkHost host) "
                      + "   (satisfies? IWorkRun run) "
                      + "   (IWorkExecutor/work-execute executor {:leaf :compile}) "
                      + "   (IWorkStore/work-query store {:query/type :run}) "
                      + "   (IWorkStore/work-transact store {:transition/run-id \"run-1\"}) "
                      + "   (satisfies? IWorkExecutor executor) "
                      + "   (satisfies? IWorkStore store)]))")
              .toString());
    }
  }

  @Test
  public void legacyWorkValuesUseTypeQualifiedNativeProtocols() {
    try (Context context = Context.newBuilder(HaraLanguage.ID).build()) {
      assertEquals(
          "[{:op :pure} \"run-legacy\"]",
          context
              .eval(
                  HaraLanguage.ID,
                  "(ns work.protocol.legacy "
                      + "(:require [work.base.model :as base])) "
                      + "(let [work (base/work-value {:op :pure}) "
                      + "      reference (base/work-reference \"run-legacy\")] "
                      + "  [(IWork/work-spec work) (IWorkRef/work-id reference)])")
              .toString());
    }
  }
}
