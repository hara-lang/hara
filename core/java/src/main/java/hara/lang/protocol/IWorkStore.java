package hara.lang.protocol;

/** Queries and atomically journals managed Work execution state. */
public interface IWorkStore {
  Object workQuery(Object query);

  Object workTransact(Object transition);
}
