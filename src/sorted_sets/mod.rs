use crate::types::TraitFuture;

pub trait SortedSet<'sorted_set, Element, Weight> {
    fn set_weight(element: Element, weight: Weight) -> TraitFuture<'sorted_set, ()>;
    fn remove(element: Element) -> TraitFuture<'sorted_set, ()>;
    fn get(from: u32, to: u32) -> TraitFuture<'sorted_set, dyn Iterator<Item = Element>>;
}
