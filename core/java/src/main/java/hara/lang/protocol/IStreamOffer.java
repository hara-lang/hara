package hara.lang.protocol;

/** Non-blocking writable stream capability. */
public interface IStreamOffer {
  boolean offer(Object value);
}
