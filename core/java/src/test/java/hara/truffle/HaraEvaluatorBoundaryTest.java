package hara.truffle;

import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;

import java.lang.reflect.Field;
import java.lang.reflect.Method;
import java.util.Arrays;
import java.util.Set;
import java.util.stream.Collectors;
import org.junit.Test;

public class HaraEvaluatorBoundaryTest {
  @Test
  public void evaluatorIsInternalAndExposesOnlySourceAndFormExecution() {
    assertFalse(java.lang.reflect.Modifier.isPublic(Evaluator.class.getModifiers()));
    Set<String> methods =
        Arrays.stream(Evaluator.class.getDeclaredMethods())
            .map(Method::getName)
            .collect(Collectors.toSet());
    assertTrue(methods.contains("evalSource"));
    assertTrue(methods.contains("evalForm"));
  }

  @Test
  public void evaluatorHasNoKernelSessionMountOrNamespaceRegistryField() {
    assertTrue(
        Arrays.stream(HaraContext.class.getDeclaredFields())
            .anyMatch(field -> field.getType() == Evaluator.class));
    for (Field field : Evaluator.class.getDeclaredFields()) {
      String type = field.getType().getName();
      assertFalse(type, type.contains("Kernel"));
      assertFalse(type, type.contains("Session"));
      assertFalse(type, type.contains("Mount"));
      assertFalse(type, type.contains("Namespace"));
    }
  }
}
