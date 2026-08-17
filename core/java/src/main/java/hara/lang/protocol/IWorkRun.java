package hara.lang.protocol;

/** Live work run with asynchronous result, events, and cancellation. */
public interface IWorkRun extends IWorkRef, IClosed {
  Object workStatus();

  IPromise workResult();

  IStream workEvents(Object options);

  IPromise workCancel(Object reason);
}
