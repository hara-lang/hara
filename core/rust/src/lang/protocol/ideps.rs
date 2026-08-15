pub trait IDeps<K, E> {
    type Ids: Iterator<Item = K>;

    fn get_entry(&self, id: &K) -> Option<E>;
    fn get_deps(&self, id: &K) -> Self::Ids;
    fn list_entries(&self) -> Self::Ids;
}
