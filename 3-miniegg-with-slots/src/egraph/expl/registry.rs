use std::sync::{Arc, Mutex};
use crate::*;

#[derive(Clone, Default)]
pub struct ProofRegistry(Arc<Mutex<HashMap<Equation, ProvenEq>>>);

fn normalize_eq(eq: &Equation) -> Equation {
    let mut theta = SlotMap::new();
    for x in eq.l.slots() {
        theta.insert(x, Slot::new(theta.len()));
    }
    for x in eq.r.slots() {
        if !theta.contains_key(x) {
            theta.insert(x, Slot::new(theta.len()));
        }
    }
    eq.apply_slotmap(&theta)
}

impl ProofRegistry {
    // will not actually insert if the registry already has a proof for it.
    pub fn insert(&self, peq: ProvenEq) -> ProvenEq {
        let eq = normalize_eq(&peq.equ());

        let mut handle = self.0.lock().unwrap();

        if let Some(x) = handle.get(&eq) {
            return x.clone();
        }

        // TODO add special cases for proofs of the form x=x, and symmetry.

        handle.insert(eq, peq.clone());
        peq
    }
}

