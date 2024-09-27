use crate::*;

#[track_caller]
pub fn prove_explicit(l: &AppliedId, r: &AppliedId, j: Option<String>) -> ProvenEq {
    let eq = Equation { l: l.clone(), r: r.clone() };
    ExplicitProof(j).check(&eq)
}

#[track_caller]
pub fn prove_reflexivity(id: &AppliedId) -> ProvenEq {
    let eq = Equation { l: id.clone(), r: id.clone() };
    ReflexivityProof.check(&eq)
}

#[track_caller]
pub fn prove_symmetry(x: ProvenEq) -> ProvenEq {
    let eq = Equation { l: x.r.clone(), r: x.l.clone() };
    SymmetryProof(x).check(&eq)
}

#[track_caller]
pub fn prove_transitivity(x: ProvenEq, y: ProvenEq) -> ProvenEq {
    let eq1 = x.clone();
    let eq2 = y.clone();
    let theta = match_app_id(&eq2.l, &eq1.r);
    let a = eq1.l.clone();
    let c = eq2.r.apply_slotmap_fresh(&theta);
    let eq = Equation { l: a, r: c };

    TransitivityProof(x.clone(), y.clone()).check(&eq)
}

#[track_caller]
pub fn prove_congruence<L: Language>(l: &AppliedId, r: &AppliedId, child_proofs: Vec<ProvenEq>, eg: &EGraph<L>) -> ProvenEq {
    let eq = Equation { l: l.clone(), r: r.clone() };
    CongruenceProof(child_proofs).check(&eq, eg)
}

impl<L: Language> EGraph<L> {
    fn disassociation_necessary(&self, peq: &ProvenEq) -> bool {
        let l_rev = peq.l.m.inverse();
        let r_rev = peq.r.m.inverse();
        let l_slots = self.slots(peq.l.id);
        let r_slots = self.slots(peq.r.id);
        for s in &peq.l.slots() & &peq.r.slots() {
            if !l_slots.contains(&l_rev[s]) { return true; }
            if !r_slots.contains(&r_rev[s]) { return true; }
        }

        false
    }

    fn get_redundancy_proof(&self, i: Id) -> ProvenEq {
        let (leader, prf) = self.proven_unionfind_get(i);
        let red_prf = self.classes[&leader.id].redundancy_proof.clone();
        let inv_prf = prove_symmetry(prf.clone());
        let out = prove_transitivity(prf, prove_transitivity(red_prf, inv_prf));
        out
    }

    pub fn disassociate_proven_eq(&self, peq: ProvenEq) -> ProvenEq {
        if self.disassociation_necessary(&peq) {
            let mut peq = peq;
            let x = self.get_redundancy_proof(peq.l.id);
            let y = self.get_redundancy_proof(peq.r.id);
            peq = prove_transitivity(x, peq);
            peq = prove_transitivity(peq, y);

            peq
        } else {
            peq
        }
    }

    #[track_caller]
    pub fn prove_explicit(&self, l: &AppliedId, r: &AppliedId, j: Option<String>) -> ProvenEq { self.disassociate_proven_eq(prove_explicit(l, r, j)) }

    #[track_caller]
    pub fn prove_reflexivity(&self, id: &AppliedId) -> ProvenEq { self.disassociate_proven_eq(prove_reflexivity(id)) }

    #[track_caller]
    pub fn prove_symmetry(&self, x: ProvenEq) -> ProvenEq { self.disassociate_proven_eq(prove_symmetry(x)) }

    #[track_caller]
    pub fn prove_transitivity(&self, x: ProvenEq, y: ProvenEq) -> ProvenEq { self.disassociate_proven_eq(prove_transitivity(x, y)) }

    #[track_caller]
    pub fn prove_congruence(&self, l: &AppliedId, r: &AppliedId, child_proofs: Vec<ProvenEq>) -> ProvenEq { self.disassociate_proven_eq(prove_congruence(l, r, child_proofs, self)) }
}
