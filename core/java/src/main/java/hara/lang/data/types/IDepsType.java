package hara.lang.data.types;

import hara.lang.protocol.IContext;

import java.util.Iterator;

/**
 * Native extension point for dependency-backed values.
 *
 * <p>Portable map contexts are handled by {@code std.foundation} using per-entry values of the
 * form {@code {:entry value :deps ids}}. The runtimes intentionally provide no built-in native
 * implementor.
 */
public interface IDepsType<K, E> {
  E getEntry(IContext ctx, K id);

  ISetType<K> getDeps(IContext ctx, K id);

  Iterator<K> listEntries(IContext ctx);
}
