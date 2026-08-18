package hara.lang.protocol;

/** Executes one leaf request from the Work algebra. */
public interface IWorkExecutor {
  Object workExecute(Object request);
}
