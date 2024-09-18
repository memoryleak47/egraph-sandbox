use crate::*;

use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Equation {
    pub lhs: AppliedId,
    pub rhs: AppliedId,
}

#[derive(Debug, Clone)]
pub struct ProvenEq {
    // fields are intentionally private so that only "add_proof" can construct instances for it.
    eq: Equation,
    proof: Proof,
}

#[derive(Debug, Clone)]
pub enum Proof {
    Explicit(/*justification: */ Option<String>),
    Rename(Arc<ProvenEq>, SlotMap),
}

impl<L: Language> EGraph<L> {
    pub fn prove(&self, eq: Equation, proof: Proof) -> Option<Arc<ProvenEq>> {
        self.check_proof(&eq, &proof)?;

        Some(Arc::new(ProvenEq { eq, proof }))
    }

    pub fn check_proof(&self, eq: &Equation, proof: &Proof) -> Option<()> {
        todo!()
    }
}
