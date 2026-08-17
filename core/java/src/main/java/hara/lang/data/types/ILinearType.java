package hara.lang.data.types;

import hara.lang.protocol.*;

public interface ILinearType<E>
    extends IColl<E>,
        IPeekFirst<E>,
        IPeekLast<E>,
        ICons<E>,
        IConj<E>,
        INth<E>,
        ICount {

  @Override
  default ICons<E> cons(E e) {
    hara.lang.data.Seq<E> tail = hara.lang.data.Seq.create(iterator());
    return new hara.lang.data.Cons<>(null, e, tail);
  }

  @SuppressWarnings("unchecked")
  @Override
  default ILinearType<E> conj(E e) {
    return (ILinearType<E>) ((IPushLast<E>) this).pushLast(e);
  }

  @Override
  default String startString() {
    return "[";
  }

  @Override
  default String endString() {
    return "]";
  }
}
