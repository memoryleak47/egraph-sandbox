use crate::*;

#[derive(Debug, Clone)]
pub struct Equation {
    lhs: AppliedId,
    rhs: AppliedId,
}

#[derive(Debug, Clone)]
pub struct ProvenEq {
    eq: Equation,
    proof: Proof,
}

#[derive(Debug, Clone)]
pub enum Proof {
    Explicit(/*justification: */ Option<String>),
    Rename(Arc<ProvenEq>, SlotMap),
}
