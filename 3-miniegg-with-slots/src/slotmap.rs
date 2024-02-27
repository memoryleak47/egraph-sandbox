use crate::*;
use std::ops::Index;

pub struct SlotMap {
    map: Vec<(Slot, Slot)>,
}

impl SlotMap {
    pub fn new() -> Self {
        SlotMap { map: Default::default() }
    }

    fn search(&self, l: Slot) -> Result<usize, usize> {
        self.map.binary_search_by_key(&l, |(x, _)| *x)
    }

    pub fn insert(&mut self, l: Slot, r: Slot) {
        match self.search(l) {
            Ok(i) => { self.map[i] = (l, r); },
            Err(i) => {
                self.map.insert(i, (l, r));
            },
        }
    }

    pub fn get(&self, l: Slot) -> Option<Slot> {
        self.search(l).ok().map(|i| self.map[i].1)
    }
}

impl Index<Slot> for SlotMap {
    type Output = Slot;

    fn index(&self, l: Slot) -> &Slot {
        let i = self.search(l).unwrap();

        &self.map[i].1
    }
}

#[test]
fn test_slotmap() {
    let mut m: SlotMap = SlotMap::new();
    m.insert(Slot(3), Slot(7));
    m.insert(Slot(2), Slot(7));
    m.insert(Slot(3), Slot(8));
    m.insert(Slot(4), Slot(7));
    assert_eq!(m[Slot(3)], Slot(8));
}
