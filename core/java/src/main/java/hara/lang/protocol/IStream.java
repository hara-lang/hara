package hara.lang.protocol;

/** Asynchronous pull source. A fulfilled null value denotes end-of-stream. */
public interface IStream extends IClose {
  Object next();
}
