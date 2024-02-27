use crate::*;
use std::ops::Index;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct SlotMap {
    // if (l, r) in map, then there is no (l, r') in map. Each key is uniquely contained.
    // Also: map is sorted by their keys.
    map: Vec<(Slot, Slot)>,
}

impl SlotMap {
    pub fn new() -> Self {
        SlotMap { map: Default::default() }
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    fn search(&self, l: Slot) -> Result<usize, usize> {
        self.map.binary_search_by_key(&l, |(x, _)| *x)
    }

    pub fn contains(&self, k: Slot) -> bool {
        self.get(k).is_some()
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

    pub fn iter(&self) -> impl Iterator<Item=(Slot, Slot)> + '_ {
        self.map.iter().copied()
    }

    pub fn keys(&self) -> impl Iterator<Item=Slot> + '_ {
        self.iter().map(|(x, _)| x)
    }

    pub fn values(&self) -> impl Iterator<Item=Slot> + '_ {
        self.iter().map(|(_, x)| x)
    }

    pub fn inverse(&self) -> SlotMap {
        assert!(self.is_bijection());

        let mut out = Self::new();
        for (x, y) in self.iter() {
            out.insert(y, x);
        }

        out
    }

    pub fn is_bijection(&self) -> bool {
        let mut found = HashSet::new();

        for (_, x) in self.iter() {
            if found.contains(&x) {
                return false;
            }

            found.insert(x);
        }

        true
    }

    pub fn compose_all(&self, other: &SlotMap) -> SlotMap {
        let mut out = SlotMap::new();
        for (x, y) in self.iter() {
            out.insert(x, other[y]);
        }
        out
    }

    pub fn compose(&self, other: &SlotMap) -> SlotMap {
        let mut out = SlotMap::new();
        for (x, y) in self.iter() {
            if let Some(z) = other.get(y) {
                out.insert(x, z);
            }
        }
        out
    }

    // checks invariants.
    fn inv(&self) {
        // sortedness.
        let mut sorted = self.map.clone();
        sorted.sort_by_key(|(l, _)| *l);
        assert_eq!(&self.map, &sorted);

        // left-uniqueness.
        let mut found = HashSet::new();
        for &(x, _) in self.map.iter() {
            assert!(!found.contains(&x));
            found.insert(x);
        }
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

    m.inv();
}
