use crate::*;

use std::ops::Index;
use std::hash::{Hash, Hasher};

pub trait Permutation: Index<Slot, Output=Slot> + Clone + Eq + Hash {
    fn iter(&self) -> impl Iterator<Item=(Slot, Slot)>;
    fn compose(&self, other: &Self) -> Self;
    fn inverse(&self) -> Self;
}

impl Permutation for Perm {
    fn iter(&self) -> impl Iterator<Item=(Slot, Slot)> { Self::iter(self) }
    fn compose(&self, other: &Self) -> Self { Self::compose(self, other) }
    fn inverse(&self) -> Self { Self::inverse(self) }
}

#[derive(Clone, Debug)]
pub struct ProvenPerm(Perm, ProvenEq);

impl PartialEq for ProvenPerm {
    fn eq(&self, other: &Self) -> bool { self.0 == other.0 }
}

impl Eq for ProvenPerm { }

impl Hash for ProvenPerm {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.0.hash(hasher);
    }
}

impl Permutation for ProvenPerm {
    fn iter(&self) -> impl Iterator<Item=(Slot, Slot)> { self.0.iter() }
    fn compose(&self, other: &Self) -> Self {
        let map = self.0.compose(&other.0);
        let prf = prove_transitivity(self.1.clone(), other.1.clone());
        ProvenPerm(map, prf)
    }

    fn inverse(&self) -> Self {
        let map = self.0.inverse();
        let prf = prove_symmetry(self.1.clone());
        ProvenPerm(map, prf)
    }
}

impl ProvenPerm {
    pub fn identity<L: Language>(i: Id, eg: &EGraph<L>) -> Self {
        let slots = &eg.slots(i);
        let map = Perm::identity(slots);

        let syn_slots = &eg.syn_slots(i);
        let identity = SlotMap::identity(syn_slots);
        let app_id = eg.mk_applied_id(i, identity);
        let prf = prove_reflexivity(&app_id);
        ProvenPerm(map, prf)
    }
}


impl Index<Slot> for ProvenPerm {
    type Output = Slot;

    fn index(&self, s: Slot) -> &Slot {
        self.0.index(s)
    }
}
