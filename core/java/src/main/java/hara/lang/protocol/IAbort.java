package hara.lang.protocol;

/** Exceptional stream termination capability. */
public interface IAbort {
  Object abort(Object error);
}
