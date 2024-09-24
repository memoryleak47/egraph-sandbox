use crate::*;

use std::ops::Index;
use std::hash::{Hash, Hasher};

pub trait Permutation: Index<Slot, Output=Slot> + Clone + Eq + Hash {
    fn iter(&self) -> impl Iterator<Item=(Slot, Slot)>;
    fn compose(&self, other: &Self) -> Self;
    fn inverse(&self) -> Self;

    fn to_slotmap(&self) -> SlotMap {
        self.iter().collect()
    }
}

impl Permutation for Perm {
    fn iter(&self) -> impl Iterator<Item=(Slot, Slot)> { Self::iter(self) }
    fn compose(&self, other: &Self) -> Self { Self::compose(self, other) }
    fn inverse(&self) -> Self { Self::inverse(self) }
}

#[derive(Clone, Debug)]
pub struct ProvenPerm(pub Perm, pub ProvenEq);

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
        if CHECKS {
            assert_eq!(self.1.l.id, self.1.r.id);
            assert_eq!(other.1.l.id, other.1.r.id);
            assert_eq!(self.1.l.id, other.1.l.id);
        }
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
    pub fn identity(i: Id, slots: &HashSet<Slot>, syn_slots: &HashSet<Slot>) -> Self {
        let map = Perm::identity(slots);

        let identity = SlotMap::identity(syn_slots);
        let app_id = AppliedId::new(i, identity);
        let prf = prove_reflexivity(&app_id);
        ProvenPerm(map, prf)
    }

    pub fn check<L: Language>(&self, eg: &EGraph<L>) {
        let id = self.1.l.id;
        let slots = eg.slots(id);
        let syn_slots = eg.syn_slots(id);

        assert_eq!(id, self.1.l.id);
        assert_eq!(id, self.1.r.id);
        assert_eq!(&self.0.keys(), &slots);
        assert_eq!(&self.0.values(), &slots);
        assert_eq!(&self.1.l.m.keys(), &syn_slots);
        assert_eq!(&self.1.l.m.values(), &syn_slots);
        assert_eq!(&self.1.r.m.keys(), &syn_slots);
        assert_eq!(&self.1.r.m.values(), &syn_slots);
        assert!(self.0.is_perm());

        let eq = Equation { l: eg.mk_identity_applied_id(id), r: eg.mk_applied_id(id, self.0.clone()) };
        match_equation(&eq, &self.1).unwrap();
    }
}

impl Index<Slot> for ProvenPerm {
    type Output = Slot;

    fn index(&self, s: Slot) -> &Slot {
        self.0.index(s)
    }
}
