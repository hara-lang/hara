package hara.truffle;

import hara.lang.base.G;
import hara.lang.data.Keyword;
import hara.lang.data.Symbol;
import hara.lang.data.types.ILinearType;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.HashSet;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Map.Entry;
import java.util.Set;

/** Canonical compiler-facing interpretation of portable HAL schema forms. */
public final class HalcSchema {
  private HalcSchema() {}

  public sealed interface Type
      permits Primitive,
          Reference,
          Union,
          VectorType,
          Tuple,
          MapType,
          FunctionType,
          EnumType,
          Extension,
          Unknown {}

  public record Primitive(String name) implements Type {}

  public record Reference(String name) implements Type {}

  public record Union(List<Type> types) implements Type {
    public Union {
      types = List.copyOf(types);
    }
  }

  public record VectorType(Type item) implements Type {}

  public record Tuple(List<Type> items) implements Type {
    public Tuple {
      items = List.copyOf(items);
    }
  }

  public record Field(Object name, Type type) {}

  public record MapType(List<Field> fields) implements Type {
    public MapType {
      fields = List.copyOf(fields);
    }
  }

  public record Function(List<Type> fixed, Type rest, Type output) {
    public Function {
      fixed = List.copyOf(fixed);
    }
  }

  public record FunctionType(List<Function> arities) implements Type {
    public FunctionType {
      arities = List.copyOf(arities);
    }
  }

  public record EnumType(List<Object> values) implements Type {
    public EnumType {
      values = List.copyOf(values);
    }
  }

  /** Retained for artifact compatibility; strict normalization never creates it. */
  public record Extension(String head, List<Object> arguments) implements Type {
    public Extension {
      arguments = List.copyOf(arguments);
    }
  }

  public record Unknown(Object surface) implements Type {}

  /** Strictly normalizes one portable schema surface form. */
  public static Type normalize(Object schema) {
    if (schema instanceof Keyword keyword) {
      String primitive = canonicalPrimitive(keyword.getName());
      if (primitive == null) {
        throw invalid(
            "unsupported-primitive", "unsupported schema primitive: :" + keyword.getName());
      }
      return new Primitive(primitive);
    }

    if (schema instanceof hara.lang.data.List<?> reference
        && reference.count() > 0
        && reference.nth(0) instanceof Symbol operator
        && operator.getNamespace() == null
        && "var".equals(operator.getName())) {
      return normalizeReference(reference);
    }

    ILinearType<?> vector = vector(schema);
    if (vector == null) {
      throw invalid("unsupported-value", "unsupported schema value: " + G.display(schema));
    }
    if (vector.count() == 0) {
      throw invalid("empty-schema", "schema vector cannot be empty");
    }
    if (!(vector.nth(0) instanceof Keyword head)) {
      throw invalid("invalid-head", "schema vector head must be a keyword");
    }

    List<Object> arguments = values(vector, 1);
    return switch (head.getName()) {
      case "or" -> normalizeUnionSurfaces(arguments);
      case "maybe" -> {
        requireCount(head.getName(), arguments, 1);
        yield normalizeUnionTypes(
            List.of(normalize(arguments.get(0)), new Primitive("nil")));
      }
      case "vector" -> {
        requireCount(head.getName(), arguments, 1);
        yield new VectorType(normalize(arguments.get(0)));
      }
      case "tuple" -> new Tuple(normalizeAll(arguments));
      case "map" -> normalizeMap(arguments);
      case "fn" -> new FunctionType(List.of(normalizeFunction(vector)));
      case "function" -> normalizeFunctionSet(arguments);
      case "enum" -> normalizeEnum(arguments);
      default -> throw invalid(
          "unsupported-form", "unsupported schema form: :" + head.getName());
    };
  }

  private static Type normalizeReference(hara.lang.data.List<?> reference) {
    if (reference.count() != 2) {
      throw invalid(
          "invalid-reference", "named schema reference must be (var qualified/Symbol)");
    }
    if (!(reference.nth(1) instanceof Symbol target)) {
      throw invalid("invalid-reference", "named schema reference must target a symbol");
    }
    if (target.getNamespace() == null
        || target.getNamespace().isEmpty()
        || target.getName().isEmpty()) {
      throw invalid(
          "unqualified-reference",
          "named schema reference is not fully qualified: " + target.display());
    }
    return new Reference(target.display());
  }

  private static FunctionType normalizeFunctionSet(List<Object> arguments) {
    if (arguments.isEmpty()) {
      throw invalid("empty-function", ":function schema requires at least one :fn schema");
    }
    List<Function> arities = new ArrayList<>();
    for (Object argument : arguments) {
      ILinearType<?> function = vector(argument);
      if (function == null
          || function.count() == 0
          || !(function.nth(0) instanceof Keyword head)
          || !"fn".equals(head.getName())) {
        throw invalid("invalid-function-member", ":function members must be :fn schemas");
      }
      arities.add(normalizeFunction(function));
    }
    return new FunctionType(arities);
  }

  private static EnumType normalizeEnum(List<Object> arguments) {
    if (arguments.isEmpty()) {
      throw invalid("empty-enum", ":enum schema requires at least one value");
    }
    List<Object> values = new ArrayList<>();
    for (Object value : arguments) {
      if (!values.contains(value)) values.add(value);
    }
    return new EnumType(values);
  }

  private static Type normalizeUnionSurfaces(List<Object> arguments) {
    if (arguments.isEmpty()) {
      throw invalid("empty-union", ":or schema requires at least one member");
    }
    return normalizeUnionTypes(normalizeAll(arguments));
  }

  private static Type normalizeUnionTypes(List<Type> types) {
    List<Type> members = new ArrayList<>();
    for (Type type : types) pushJoined(members, type);
    if (members.isEmpty()) {
      throw invalid("empty-union", ":or schema requires at least one member");
    }
    return members.size() == 1 ? members.get(0) : new Union(members);
  }

  private static MapType normalizeMap(List<Object> arguments) {
    List<Field> fields = new ArrayList<>();
    Set<Object> names = new HashSet<>();
    for (Object argument : arguments) {
      ILinearType<?> pair = vector(argument);
      if (pair == null || pair.count() != 2) {
        throw invalid("invalid-map-field", ":map schema fields must be [name type] pairs");
      }
      Object name = pair.nth(0);
      if (!names.add(name)) {
        throw invalid("duplicate-map-field", "duplicate :map schema field: " + G.display(name));
      }
      fields.add(new Field(name, normalize(pair.nth(1))));
    }
    return new MapType(fields);
  }

  private static Function normalizeFunction(ILinearType<?> function) {
    if (function.count() != 3
        || !(function.nth(0) instanceof Keyword head)
        || !"fn".equals(head.getName())) {
      throw invalid("invalid-arity", ":fn schema must be [:fn [inputs ...] output]");
    }
    ILinearType<?> inputs = vector(function.nth(1));
    if (inputs == null) {
      throw invalid("invalid-function-inputs", ":fn schema inputs must be a vector");
    }
    List<Type> fixed = new ArrayList<>();
    Type rest = null;
    int index = 0;
    while (index < inputs.count()) {
      Object input = inputs.nth(index);
      if (input instanceof Symbol marker
          && marker.getNamespace() == null
          && "&".equals(marker.getName())) {
        if (rest != null || index + 2 != inputs.count()) {
          throw invalid(
              "invalid-function-rest", ":fn schema & must precede exactly one rest type");
        }
        rest = normalize(inputs.nth(index + 1));
        index += 2;
      } else {
        fixed.add(normalize(input));
        index++;
      }
    }
    return new Function(fixed, rest, normalize(function.nth(2)));
  }

  /** Resolves named references without evaluating code; cycles remain references. */
  public static Type resolve(Type schema, Map<String, Type> definitions) {
    return resolve(schema, definitions, new HashSet<>());
  }

  private static Type resolve(
      Type schema, Map<String, Type> definitions, Set<String> visited) {
    if (schema == null) return unknown();
    if (schema instanceof Reference reference) {
      if (visited.contains(reference.name())) return reference;
      Type target = definitions.get(reference.name());
      if (target == null) return reference;
      Set<String> nested = new HashSet<>(visited);
      nested.add(reference.name());
      return resolve(target, definitions, nested);
    }
    if (schema instanceof Union union) {
      List<Type> members = new ArrayList<>();
      for (Type member : union.types()) members.add(resolve(member, definitions, visited));
      return normalizeUnionTypes(members);
    }
    if (schema instanceof VectorType vector) {
      return new VectorType(resolve(vector.item(), definitions, visited));
    }
    if (schema instanceof Tuple tuple) {
      List<Type> items = new ArrayList<>();
      for (Type item : tuple.items()) items.add(resolve(item, definitions, visited));
      return new Tuple(items);
    }
    if (schema instanceof MapType map) {
      List<Field> fields = new ArrayList<>();
      for (Field field : map.fields()) {
        fields.add(new Field(field.name(), resolve(field.type(), definitions, visited)));
      }
      return new MapType(fields);
    }
    if (schema instanceof FunctionType functions) {
      List<Function> arities = new ArrayList<>();
      for (Function function : functions.arities()) {
        List<Type> fixed = new ArrayList<>();
        for (Type input : function.fixed()) fixed.add(resolve(input, definitions, visited));
        Type rest =
            function.rest() == null ? null : resolve(function.rest(), definitions, visited);
        arities.add(
            new Function(fixed, rest, resolve(function.output(), definitions, visited)));
      }
      return new FunctionType(arities);
    }
    return schema;
  }

  /** Returns all named references in a normalized schema. */
  public static Set<String> references(Type schema) {
    Set<String> output = new HashSet<>();
    collectReferences(schema, output);
    return Set.copyOf(output);
  }

  private static void collectReferences(Type schema, Set<String> output) {
    if (schema instanceof Reference reference) {
      output.add(reference.name());
    } else if (schema instanceof Union union) {
      for (Type member : union.types()) collectReferences(member, output);
    } else if (schema instanceof VectorType vector) {
      collectReferences(vector.item(), output);
    } else if (schema instanceof Tuple tuple) {
      for (Type item : tuple.items()) collectReferences(item, output);
    } else if (schema instanceof MapType map) {
      for (Field field : map.fields()) collectReferences(field.type(), output);
    } else if (schema instanceof FunctionType functions) {
      for (Function function : functions.arities()) {
        for (Type input : function.fixed()) collectReferences(input, output);
        if (function.rest() != null) collectReferences(function.rest(), output);
        collectReferences(function.output(), output);
      }
    }
  }

  /** Symmetric domain-overlap relation used for conservative checks. */
  public static boolean compatible(Type expected, Type actual) {
    if (expected == null || actual == null) return true;
    if (expected.equals(actual)) return true;
    if (expected instanceof Unknown || actual instanceof Unknown) return true;
    if (expected instanceof Reference || actual instanceof Reference) return true;
    if (expected instanceof Primitive left && actual instanceof Primitive right) {
      return primitiveCompatible(left.name(), right.name());
    }
    if (expected instanceof Union union) {
      for (Type member : union.types()) if (compatible(member, actual)) return true;
      return false;
    }
    if (actual instanceof Union union) {
      for (Type member : union.types()) if (compatible(expected, member)) return true;
      return false;
    }
    if (expected instanceof VectorType left && actual instanceof VectorType right) {
      return compatible(left.item(), right.item());
    }
    if (expected instanceof Tuple left && actual instanceof Tuple right) {
      if (left.items().size() != right.items().size()) return false;
      for (int index = 0; index < left.items().size(); index++) {
        if (!compatible(left.items().get(index), right.items().get(index))) return false;
      }
      return true;
    }
    if (expected instanceof MapType left && actual instanceof MapType right) {
      Map<Object, Type> rightFields = fieldsByName(right.fields());
      for (Field field : left.fields()) {
        Type candidate = rightFields.get(field.name());
        if (candidate != null && !compatible(field.type(), candidate)) return false;
      }
      return true;
    }
    if (expected instanceof EnumType left && actual instanceof EnumType right) {
      for (Object value : left.values()) if (right.values().contains(value)) return true;
      return false;
    }
    return false;
  }

  /** Directional relation: every actual value must fit the expected domain. */
  public static boolean assignable(Type expected, Type actual) {
    if (expected == null || actual == null) return true;
    if (expected.equals(actual)) return true;
    if (expected instanceof Unknown || actual instanceof Unknown) return true;
    if (expected instanceof Primitive primitive && "any".equals(primitive.name())) return true;
    if (expected instanceof Reference || actual instanceof Reference) return true;

    if (expected instanceof Union left) {
      if (actual instanceof Union right) {
        for (Type actualMember : right.types()) {
          boolean accepted = false;
          for (Type expectedMember : left.types()) {
            if (assignable(expectedMember, actualMember)) {
              accepted = true;
              break;
            }
          }
          if (!accepted) return false;
        }
        return true;
      }
      for (Type member : left.types()) if (assignable(member, actual)) return true;
      return false;
    }
    if (actual instanceof Union right) {
      for (Type member : right.types()) if (!assignable(expected, member)) return false;
      return true;
    }
    if (expected instanceof Primitive left && actual instanceof Primitive right) {
      return left.name().equals(right.name())
          || ("num".equals(left.name()) && isNumericPrimitive(right.name()));
    }
    if (expected instanceof VectorType left && actual instanceof VectorType right) {
      return assignable(left.item(), right.item());
    }
    if (expected instanceof Tuple left && actual instanceof Tuple right) {
      if (left.items().size() != right.items().size()) return false;
      for (int index = 0; index < left.items().size(); index++) {
        if (!assignable(left.items().get(index), right.items().get(index))) return false;
      }
      return true;
    }
    if (expected instanceof MapType left && actual instanceof MapType right) {
      Map<Object, Type> rightFields = fieldsByName(right.fields());
      for (Field field : left.fields()) {
        Type candidate = rightFields.get(field.name());
        if (candidate == null || !assignable(field.type(), candidate)) return false;
      }
      return true;
    }
    if (expected instanceof EnumType left && actual instanceof EnumType right) {
      return left.values().containsAll(right.values());
    }
    if (expected instanceof FunctionType left && actual instanceof FunctionType right) {
      return functionsAssignable(left, right);
    }
    return false;
  }

  private static boolean functionsAssignable(FunctionType expected, FunctionType actual) {
    for (Function expectedArity : expected.arities()) {
      boolean accepted = false;
      for (Function actualArity : actual.arities()) {
        if (functionAssignable(expectedArity, actualArity)) {
          accepted = true;
          break;
        }
      }
      if (!accepted) return false;
    }
    return true;
  }

  private static boolean functionAssignable(Function expected, Function actual) {
    if (expected.fixed().size() != actual.fixed().size()) return false;
    if ((expected.rest() == null) != (actual.rest() == null)) return false;
    for (int index = 0; index < expected.fixed().size(); index++) {
      if (!assignable(actual.fixed().get(index), expected.fixed().get(index))) return false;
    }
    if (expected.rest() != null && !assignable(actual.rest(), expected.rest())) return false;
    return assignable(expected.output(), actual.output());
  }

  private static Map<Object, Type> fieldsByName(List<Field> fields) {
    Map<Object, Type> output = new LinkedHashMap<>();
    for (Field field : fields) output.put(field.name(), field.type());
    return output;
  }

  private static boolean primitiveCompatible(String expected, String actual) {
    return expected.equals(actual)
        || "any".equals(expected)
        || "any".equals(actual)
        || ("num".equals(expected) && isNumericPrimitive(actual))
        || ("num".equals(actual) && isNumericPrimitive(expected));
  }

  private static boolean isNumericPrimitive(String name) {
    return "int".equals(name) || "float".equals(name) || "decimal".equals(name);
  }

  /** Conservatively infers one form without evaluating it. */
  public static Type infer(
      Object form, Map<String, Type> environment, Map<String, Type> functions) {
    return inferExpression(form, environment, functions);
  }

  private static Type inferExpression(
      Object form, Map<String, Type> environment, Map<String, Type> functions) {
    if (form == null) return new Primitive("nil");
    if (form instanceof Boolean) return new Primitive("bool");
    if (form instanceof Byte || form instanceof Short || form instanceof Integer || form instanceof Long)
      return new Primitive("int");
    if (form instanceof Float || form instanceof Double) return new Primitive("float");
    if (form instanceof java.math.BigInteger) return new Primitive("int");
    if (form instanceof java.math.BigDecimal) return new Primitive("decimal");
    if (form instanceof Character) return new Primitive("char");
    if (form instanceof java.util.regex.Pattern) return new Primitive("regex");
    if (form instanceof String) return new Primitive("str");
    if (form instanceof Keyword) return new Primitive("keyword");
    if (form instanceof Symbol symbol) {
      Type local = environment.get(symbol.display());
      if (local == null) local = environment.get(symbol.getName());
      return local == null ? unknown(symbol) : local;
    }

    ILinearType<?> vector = vector(form);
    if (vector != null) {
      List<Type> members = new ArrayList<>();
      for (int index = 0; index < vector.count(); index++) {
        members.add(inferExpression(vector.nth(index), environment, functions));
      }
      return new VectorType(joinTypes(members));
    }
    if (form instanceof hara.lang.data.types.IMapType<?, ?> map) {
      List<Field> fields = new ArrayList<>();
      for (Object item : map) {
        Entry<?, ?> entry = (Entry<?, ?>) item;
        fields.add(
            new Field(
                entry.getKey(), inferExpression(entry.getValue(), environment, functions)));
      }
      return new MapType(fields);
    }
    if (form instanceof hara.lang.data.types.ISetType<?>) return new Primitive("set");
    if (!(form instanceof hara.lang.data.List<?> list)) return unknown(form);
    if (list.count() == 0) return new Primitive("list");
    if (!(list.nth(0) instanceof Symbol operator)) return unknown(form);
    return inferList(list, operator, environment, functions);
  }

  private static Type inferList(
      hara.lang.data.List<?> list,
      Symbol operator,
      Map<String, Type> environment,
      Map<String, Type> functions) {
    String name = operator.getName();
    switch (name) {
      case "quote":
        return list.count() >= 2 ? inferLiteral(list.nth(1)) : unknown(list);
      case "do": {
        Type output = new Primitive("nil");
        for (int index = 1; index < list.count(); index++) {
          output = inferExpression(list.nth(index), environment, functions);
        }
        return output;
      }
      case "if": {
        List<Type> branches = new ArrayList<>();
        for (int index = 2; index < list.count(); index++) {
          branches.add(inferExpression(list.nth(index), environment, functions));
        }
        if (list.count() == 3) branches.add(new Primitive("nil"));
        return joinTypes(branches);
      }
      case "let":
      case "loop": {
        Map<String, Type> nested = new HashMap<>(environment);
        ILinearType<?> bindings = list.count() > 1 ? vector(list.nth(1)) : null;
        if (bindings == null) return unknown(list);
        for (int index = 0; index + 1 < bindings.count(); index += 2) {
          if (bindings.nth(index) instanceof Symbol binding) {
            nested.put(
                binding.getName(),
                inferExpression(bindings.nth(index + 1), nested, functions));
          }
        }
        Type output = new Primitive("nil");
        for (int index = 2; index < list.count(); index++) {
          output = inferExpression(list.nth(index), nested, functions);
        }
        return output;
      }
      case "+":
      case "-":
      case "*":
      case "%":
      case "mod":
        return inferNumeric(list, environment, functions, false);
      case "/":
        return inferNumeric(list, environment, functions, true);
      case "=":
      case "not=":
      case "<":
      case "<=":
      case ">":
      case ">=":
      case "identical?":
      case "instance?":
      case "nil?":
      case "some?":
        return new Primitive("bool");
      case "count":
        return new Primitive("int");
      case "str":
        return new Primitive("str");
      case "keyword":
        return new Primitive("keyword");
      case "symbol":
        return new Primitive("symbol");
      case "vector": {
        List<Type> members = new ArrayList<>();
        for (int index = 1; index < list.count(); index++) {
          members.add(inferExpression(list.nth(index), environment, functions));
        }
        return new VectorType(joinTypes(members));
      }
      case "hash-map": {
        List<Field> fields = new ArrayList<>();
        for (int index = 1; index + 1 < list.count(); index += 2) {
          fields.add(
              new Field(
                  list.nth(index),
                  inferExpression(list.nth(index + 1), environment, functions)));
        }
        return new MapType(fields);
      }
      default:
        return inferKnownCall(list, operator, environment, functions);
    }
  }

  private static Type inferNumeric(
      hara.lang.data.List<?> list,
      Map<String, Type> environment,
      Map<String, Type> functions,
      boolean divide) {
    List<Type> values = new ArrayList<>();
    for (int index = 1; index < list.count(); index++) {
      Type value = inferExpression(list.nth(index), environment, functions);
      if (value instanceof Unknown) return value;
      values.add(value);
    }
    if (values.isEmpty()) return unknown(list);
    Primitive number = new Primitive("num");
    for (Type value : values) if (!assignable(number, value)) return unknown(list);
    if (divide) return number;
    for (Type value : values) if (!new Primitive("int").equals(value)) return number;
    return new Primitive("int");
  }

  private static Type inferKnownCall(
      hara.lang.data.List<?> list,
      Symbol operator,
      Map<String, Type> environment,
      Map<String, Type> functions) {
    Type contract = functions.get(operator.display());
    if (contract == null) contract = functions.get(operator.getName());
    Function arity = matchingArity(contract, Math.toIntExact(list.count()) - 1);
    return arity == null ? unknown(operator) : arity.output();
  }

  private static Type inferLiteral(Object value) {
    if (value == null) return new Primitive("nil");
    if (value instanceof Boolean) return new Primitive("bool");
    if (value instanceof Byte || value instanceof Short || value instanceof Integer || value instanceof Long
        || value instanceof java.math.BigInteger) return new Primitive("int");
    if (value instanceof Float || value instanceof Double) return new Primitive("float");
    if (value instanceof java.math.BigDecimal) return new Primitive("decimal");
    if (value instanceof Character) return new Primitive("char");
    if (value instanceof java.util.regex.Pattern) return new Primitive("regex");
    if (value instanceof String) return new Primitive("str");
    if (value instanceof Keyword) return new Primitive("keyword");
    if (value instanceof Symbol) return new Primitive("symbol");
    ILinearType<?> vector = vector(value);
    if (vector != null) {
      List<Type> members = new ArrayList<>();
      for (int index = 0; index < vector.count(); index++) {
        members.add(inferLiteral(vector.nth(index)));
      }
      return new VectorType(joinTypes(members));
    }
    if (value instanceof hara.lang.data.types.IMapType<?, ?> map) {
      List<Field> fields = new ArrayList<>();
      for (Object item : map) {
        Entry<?, ?> entry = (Entry<?, ?>) item;
        fields.add(new Field(entry.getKey(), inferLiteral(entry.getValue())));
      }
      return new MapType(fields);
    }
    if (value instanceof hara.lang.data.types.ISetType<?>) return new Primitive("set");
    if (value instanceof hara.lang.data.List<?>) return new Primitive("list");
    return unknown(value);
  }

  private static Type joinTypes(List<Type> values) {
    if (values.isEmpty()) return unknown();
    List<Type> members = new ArrayList<>();
    for (Type value : values) {
      if (value instanceof Unknown) return value;
      pushJoined(members, value);
    }
    return members.size() == 1 ? members.get(0) : new Union(members);
  }

  private static Function matchingArity(Type type, int argumentCount) {
    if (!(type instanceof FunctionType functions)) return null;
    for (Function function : functions.arities()) {
      if (function.rest() == null) {
        if (function.fixed().size() == argumentCount) return function;
      } else if (function.fixed().size() <= argumentCount) {
        return function;
      }
    }
    return null;
  }

  /** Conservative body-derived function facts used by lowering tiers. */
  public static Map<String, Type> inferFunctionTypes(
      String namespace,
      Object[] forms,
      Map<String, Type> declarations,
      Map<String, Type> definitions) {
    Map<String, Type> current = Map.of();
    for (int pass = 0; pass <= forms.length; pass++) {
      Map<String, Type> next =
          inferFunctionPass(namespace, forms, declarations, definitions, current);
      if (next.equals(current)) return next;
      current = next;
    }
    return current;
  }

  private static Map<String, Type> inferFunctionPass(
      String namespace,
      Object[] forms,
      Map<String, Type> declarations,
      Map<String, Type> definitions,
      Map<String, Type> inferred) {
    Map<String, Type> output = new LinkedHashMap<>();
    Map<String, Type> functions = new HashMap<>();
    functions.putAll(inferred);
    functions.putAll(declarations);
    for (Entry<String, Type> entry : new ArrayList<>(functions.entrySet())) {
      int slash = entry.getKey().lastIndexOf('/');
      if (slash >= 0 && slash + 1 < entry.getKey().length()) {
        functions.putIfAbsent(entry.getKey().substring(slash + 1), entry.getValue());
      }
    }

    for (Object form : forms) {
      if (!(form instanceof hara.lang.data.List<?> definition) || definition.count() < 3) continue;
      if (!(definition.nth(0) instanceof Symbol operator)
          || operator.getNamespace() != null
          || !("defn".equals(operator.getName()) || "defn-".equals(operator.getName()))) continue;
      if (!(definition.nth(1) instanceof Symbol name)) continue;

      String qualified = namespace + "/" + name.getName();
      Type declared = resolve(declarations.get(qualified), definitions);
      List<Function> arities = inferDefinitionArities(definition, declared, functions);
      if (!arities.isEmpty()) output.put(qualified, new FunctionType(arities));
    }
    return Map.copyOf(output);
  }

  private static List<Function> inferDefinitionArities(
      hara.lang.data.List<?> definition, Type declared, Map<String, Type> functions) {
    int parametersAt = -1;
    for (int index = 2; index < definition.count(); index++) {
      if (vector(definition.nth(index)) != null) {
        parametersAt = index;
        break;
      }
    }

    List<Function> arities = new ArrayList<>();
    if (parametersAt >= 0) {
      arities.add(
          inferFunctionArity(
              vector(definition.nth(parametersAt)),
              definition,
              parametersAt + 1,
              Math.toIntExact(definition.count()),
              declared,
              functions));
      return arities;
    }

    for (int index = 2; index < definition.count(); index++) {
      if (!(definition.nth(index) instanceof hara.lang.data.List<?> clause)
          || clause.count() == 0
          || vector(clause.nth(0)) == null) continue;
      arities.add(
          inferFunctionArity(
              vector(clause.nth(0)),
              clause,
              1,
              Math.toIntExact(clause.count()),
              declared,
              functions));
    }
    return arities;
  }

  private static Function inferFunctionArity(
      ILinearType<?> parameters,
      ILinearType<?> body,
      int bodyStart,
      int bodyEnd,
      Type declared,
      Map<String, Type> functions) {
    int fixedCount = 0;
    boolean variadic = false;
    for (int index = 0; index < parameters.count(); index++) {
      Object parameter = parameters.nth(index);
      if (parameter instanceof Symbol marker
          && marker.getNamespace() == null
          && "&".equals(marker.getName())) {
        variadic = true;
      } else if (!variadic) {
        fixedCount++;
      }
    }

    Function declaredArity = matchingDeclaredArity(declared, fixedCount, variadic);
    Map<String, Type> environment = new HashMap<>();
    List<Type> fixed = new ArrayList<>();
    Type rest = null;
    int fixedIndex = 0;
    variadic = false;
    for (int index = 0; index < parameters.count(); index++) {
      Object parameter = parameters.nth(index);
      if (parameter instanceof Symbol marker
          && marker.getNamespace() == null
          && "&".equals(marker.getName())) {
        variadic = true;
        continue;
      }
      if (!(parameter instanceof Symbol binding)) continue;
      Type type =
          variadic
              ? declaredArity != null && declaredArity.rest() != null
                  ? declaredArity.rest()
                  : unknown(binding)
              : declaredArity != null && fixedIndex < declaredArity.fixed().size()
                  ? declaredArity.fixed().get(fixedIndex)
                  : unknown(binding);
      environment.put(binding.getName(), type);
      if (variadic) {
        rest = type;
      } else {
        fixed.add(type);
        fixedIndex++;
      }
    }

    Type output = new Primitive("nil");
    for (int index = bodyStart; index < bodyEnd; index++) {
      output = inferExpression(body.nth(index), environment, functions);
    }
    return new Function(fixed, rest, output);
  }

  private static Function matchingDeclaredArity(Type declared, int fixed, boolean variadic) {
    if (!(declared instanceof FunctionType functions)) return null;
    for (Function function : functions.arities()) {
      if (function.fixed().size() == fixed && (function.rest() != null) == variadic) return function;
    }
    return null;
  }

  private static String canonicalPrimitive(String name) {
    return switch (name) {
      case "boolean" -> "bool";
      case "number" -> "num";
      case "integer" -> "int";
      case "string" -> "str";
      case "any", "nil", "bool", "num", "int", "float", "decimal", "str", "char",
          "regex", "keyword", "symbol", "list", "vector", "map", "set", "fn", "atom",
          "bytes", "promise" -> name;
      default -> null;
    };
  }

  private static Unknown unknown() {
    return unknown(Symbol.create("?"));
  }

  private static Unknown unknown(Object surface) {
    return new Unknown(surface);
  }

  /** Canonical reader-form bridge used by the cross-runtime HBC schema codec. */
  public static Object readSurface(String source) {
    Object[] forms = HaraLanguage.readAll(source, "hbc:schema");
    if (forms.length != 1) throw invalid("invalid-surface", "schema surface must contain one form");
    return forms[0];
  }

  /** Canonical readable spelling used by the cross-runtime HBC schema codec. */
  public static String displaySurface(Object value) {
    return G.display(value);
  }

  private static List<Type> normalizeAll(List<Object> values) {
    List<Type> output = new ArrayList<>(values.size());
    for (Object value : values) output.add(normalize(value));
    return output;
  }

  private static List<Object> values(ILinearType<?> input, int start) {
    List<Object> output = new ArrayList<>(Math.toIntExact(input.count()) - start);
    for (int index = start; index < input.count(); index++) output.add(input.nth(index));
    return output;
  }

  private static ILinearType<?> vector(Object value) {
    return value instanceof ILinearType<?> linear && "[".equals(linear.startString())
        ? linear
        : null;
  }

  private static void requireCount(String head, List<Object> arguments, int expected) {
    if (arguments.size() != expected) {
      throw invalid(
          "invalid-arity",
          ":"
              + head
              + " schema expects "
              + expected
              + (expected == 1 ? " argument, got " : " arguments, got ")
              + arguments.size());
    }
  }

  private static void pushJoined(List<Type> output, Type type) {
    if (type instanceof Union union) {
      for (Type member : union.types()) pushUnique(output, member);
    } else {
      pushUnique(output, type);
    }
  }

  private static void pushUnique(List<Type> output, Type value) {
    if (!output.contains(value)) output.add(value);
  }

  private static HaraException invalid(String code, String detail) {
    return new HaraException(code + ": " + detail);
  }
}
