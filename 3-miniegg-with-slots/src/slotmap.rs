use crate::*;
use std::ops::Index;

#[derive(Clone, Hash, PartialEq, Eq)]
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

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    fn search(&self, l: Slot) -> Result<usize, usize> {
        self.map.binary_search_by_key(&l, |(x, _)| *x)
    }

    pub fn contains_key(&self, k: Slot) -> bool {
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

    pub fn keys(&self) -> HashSet<Slot> {
        self.iter().map(|(x, _)| x).collect()
    }

    pub fn values(&self) -> HashSet<Slot> {
        self.iter().map(|(_, x)| x).collect()
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

    pub fn identity(set: &HashSet<Slot>) -> SlotMap {
        let mut out = SlotMap::new();
        for &x in set {
            out.insert(x, x);
        }
        out
    }

    pub fn bijection_from_fresh_to(set: &HashSet<Slot>) -> SlotMap {
        let mut out = SlotMap::new();
        for &x in set {
            out.insert(Slot::fresh(), x);
        }
        out
    }

    pub fn remove(&mut self, x: Slot) {
        if let Ok(i) = self.search(x) {
            self.map.remove(i);
        }
    }

    pub fn from_pairs(pairs: &[(Slot, Slot)]) -> SlotMap {
        pairs.iter().copied().collect()
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

    #[track_caller]
    #[inline]
    fn index(&self, l: Slot) -> &Slot {
        let Ok(i) = self.search(l) else {
            panic!("SlotMap::index({:?}): index missing!", l);
        };

        &self.map[i].1
    }
}

impl FromIterator<(Slot, Slot)> for SlotMap {
    fn from_iter<T>(iter: T) -> Self where T: IntoIterator<Item = (Slot, Slot)> {
        let mut m = SlotMap::new();
        for (x, y) in iter.into_iter() {
            assert!(!m.contains_key(x));
            m.insert(x, y);
        }
        m
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
