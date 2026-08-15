/// Native extension point for dependency-backed values.
///
/// Portable map contexts are handled by `std.foundation` using per-entry
/// `{:entry value :deps ids}` values. The runtime intentionally provides no
/// blanket native implementation.
pub trait IDeps<K, E> {
    type Ids: Iterator<Item = K>;

    fn get_entry(&self, id: &K) -> Option<E>;
    fn get_deps(&self, id: &K) -> Self::Ids;
    fn list_entries(&self) -> Self::Ids;
}
