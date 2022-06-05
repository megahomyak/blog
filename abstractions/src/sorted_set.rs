pub struct WeightedKey<Key, Weight> {
    pub key: Key,
    pub weight: Weight,
}

pub trait SortedSet<Key, Weight> {
    type Iter: Iterator<Item = WeightedKey<Key, Weight>>;

    fn set(&mut self, key: Key, weight: Weight);
    fn remove(&mut self, key: Key);
    fn iter(&mut self, chunk_size: usize, chunk_index: usize) -> Self::Iter;
}
