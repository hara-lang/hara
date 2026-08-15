package hara.lang.protocol;

import hara.lang.data.types.ISetType;
import java.util.Iterator;

/** Context-aware dependency lookup for books, snapshots, and similar stores. */
public interface IDeps<K, E> {
  E depGet(IContext context, K key);

  ISetType<K> depEntries(IContext context, K key);

  Iterator<K> depKeys(IContext context);
}
