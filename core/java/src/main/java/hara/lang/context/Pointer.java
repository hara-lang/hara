package hara.lang.context;

import hara.lang.data.Keyword;
import hara.lang.protocol.Constant;
import hara.lang.protocol.IApplicable;
import hara.lang.protocol.ICount;
import hara.lang.protocol.IContext;
import hara.lang.protocol.IDeref;
import hara.lang.protocol.IIter;
import hara.lang.protocol.ILookup;
import hara.lang.protocol.IMetadata;
import hara.lang.protocol.IObjType;
import hara.lang.protocol.IPointer;
import java.util.AbstractMap.SimpleImmutableEntry;
import java.util.Collections;
import java.util.LinkedHashMap;
import java.util.Iterator;
import java.util.Map;
import java.util.Objects;

/** An immutable context-qualified reference descriptor. */
public final class Pointer
    implements IPointer,
        IApplicable,
        IDeref<Object>,
        ILookup<Object, Object>,
        ICount,
        IIter<Map.Entry<Object, Object>>,
        IObjType {
  private final Object context;
  private final Map<Object, Object> values;
  private final IMetadata metadata;

  public Pointer(Object context, Map<?, ?> values) {
    this(context, values, null);
  }

  private Pointer(Object context, Map<?, ?> values, IMetadata metadata) {
    if (context == null) throw new IllegalArgumentException("Context required");
    this.context = context;
    Map<Object, Object> copied = new LinkedHashMap<>();
    if (values != null) values.forEach(copied::put);
    this.values = Collections.unmodifiableMap(copied);
    this.metadata = metadata;
  }

  public Object context() {
    return context;
  }

  public Map<Object, Object> values() {
    return values;
  }

  @Override
  public Object ptrContext() {
    return context;
  }

  @Override
  public Object lookup(Object key) {
    Object value = values.get(key);
    if (value == null && key instanceof Keyword) {
      value = values.get(((Keyword) key).getName());
    }
    return value;
  }

  @Override
  public Object lookup(Object key, Object notFound) {
    Object value = lookup(key);
    return value == null && !containsKey(key) ? notFound : value;
  }

  @Override
  public Map.Entry<Object, Object> find(Object key) {
    return containsKey(key) ? new SimpleImmutableEntry<>(key, lookup(key)) : null;
  }

  @Override
  @SuppressWarnings("unchecked")
  public Iterator<Object> keys() {
    return (Iterator<Object>) (Iterator<?>) values.keySet().iterator();
  }

  @Override
  @SuppressWarnings("unchecked")
  public Iterator<Object> vals() {
    return (Iterator<Object>) (Iterator<?>) values.values().iterator();
  }

  @Override
  public long count() {
    return values.size();
  }

  @Override
  @SuppressWarnings("unchecked")
  public Iterator<Map.Entry<Object, Object>> iter() {
    return (Iterator<Map.Entry<Object, Object>>) (Iterator<?>) values.entrySet().iterator();
  }

  private boolean containsKey(Object key) {
    return values.containsKey(key)
        || (key instanceof Keyword && values.containsKey(((Keyword) key).getName()));
  }

  @Override
  public Object deref() {
    throw new IllegalStateException("Pointer deref requires the active evaluator context");
  }

  @Override
  public Object applyIn(Object runtime, Object[] args) {
    return requireRuntime(runtime).invokePtr(this, args);
  }

  @Override
  public Object transformIn(Object runtime, Object[] args) {
    return requireRuntime(runtime).transformInPtr(this, args);
  }

  @Override
  public Object transformOut(Object runtime, Object[] args, Object value) {
    return requireRuntime(runtime).transformOutPtr(this, value);
  }

  @Override
  public IMetadata meta() {
    return metadata;
  }

  @Override
  public IObjType withMeta(IMetadata meta) {
    return metadata == meta ? this : new Pointer(context, values, meta);
  }

  @Override
  public long hashCalc(Constant.HashType type) {
    return Objects.hash(context, values);
  }

  @Override
  public String display() {
    Map<Object, Object> descriptor = new LinkedHashMap<>();
    descriptor.put(Keyword.create("context"), context);
    descriptor.putAll(values);
    return "#ptr " + hara.lang.base.G.display(descriptor);
  }

  private IContext requireRuntime(Object runtime) {
    if (runtime instanceof IContext) return (IContext) runtime;
    throw new IllegalArgumentException("Pointer application requires an IContext runtime");
  }

  @Override
  public boolean equals(Object other) {
    return other instanceof Pointer pointer
        && Objects.equals(context, pointer.context)
        && Objects.equals(values, pointer.values);
  }

  @Override
  public int hashCode() {
    return Objects.hash(context, values);
  }

  @Override
  public Constant.ObjType getObjType() {
    return Constant.ObjType.CLASS;
  }

  @Override
  public String getObjName() {
    return "POINTER";
  }
}
