use crate::*;

use std::cmp::Ordering;

#[derive(PartialEq, Eq)]
pub struct WithOrd<T: Eq>(pub T, pub u64);

impl<T: Eq> PartialOrd for WithOrd<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // NOTE: it's the other way around, because we want a min-heap.
        other.1.partial_cmp(&self.1)
    }
}
impl<T: Eq> Ord for WithOrd<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(&other).unwrap()
    }
}
