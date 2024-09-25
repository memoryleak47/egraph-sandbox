use crate::*;

#[track_caller]
pub fn prove_explicit(l: &AppliedId, r: &AppliedId, j: Option<String>) -> ProvenEq {
    let eq = Equation { l: l.clone(), r: r.clone() };
    ExplicitProof(j).check(&eq).unwrap()
}

#[track_caller]
pub fn prove_reflexivity(id: &AppliedId) -> ProvenEq {
    let eq = Equation { l: id.clone(), r: id.clone() };
    ReflexivityProof.check(&eq).unwrap()
}

#[track_caller]
pub fn prove_symmetry(x: ProvenEq) -> ProvenEq {
    let eq = Equation { l: x.r.clone(), r: x.l.clone() };
    SymmetryProof(x).check(&eq).unwrap()
}

#[track_caller]
pub fn prove_transitivity(x: ProvenEq, y: ProvenEq) -> ProvenEq {
    let eq1 = x.clone();
    let eq2 = y.clone();
    let theta = match_app_id(&eq2.l, &eq1.r).unwrap();
    let a = eq1.l.clone();
    let c = eq2.r.apply_slotmap_fresh(&theta);
    let eq = Equation { l: a, r: c };

    TransitivityProof(x.clone(), y.clone()).check(&eq).unwrap()
}

#[track_caller]
pub fn prove_congruence<L: Language>(l: &AppliedId, r: &AppliedId, child_proofs: Vec<ProvenEq>, eg: &EGraph<L>) -> ProvenEq {
    let eq = Equation { l: l.clone(), r: r.clone() };
    CongruenceProof(child_proofs).check(&eq, eg).unwrap()
}

impl<L: Language> EGraph<L> {
    pub fn disassociate_proven_eq(&self, peq: ProvenEq) -> ProvenEq {
        // TODO
        peq
    }

    pub fn prove_explicit(&self, l: &AppliedId, r: &AppliedId, j: Option<String>) -> ProvenEq { self.disassociate_proven_eq(prove_explicit(l, r, j)) }
    pub fn prove_reflexivity(&self, id: &AppliedId) -> ProvenEq { self.disassociate_proven_eq(prove_reflexivity(id)) }
    pub fn prove_symmetry(&self, x: ProvenEq) -> ProvenEq { self.disassociate_proven_eq(prove_symmetry(x)) }
    pub fn prove_transitivity(&self, x: ProvenEq, y: ProvenEq) -> ProvenEq { self.disassociate_proven_eq(prove_transitivity(x, y)) }
    pub fn prove_congruence(&self, l: &AppliedId, r: &AppliedId, child_proofs: Vec<ProvenEq>) -> ProvenEq { self.disassociate_proven_eq(prove_congruence(l, r, child_proofs, self)) }
}
