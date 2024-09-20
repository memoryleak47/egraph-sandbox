use crate::*;

impl<L: Language> EGraph<L> {
    pub fn prove_explicit(&self, l: &AppliedId, r: &AppliedId, j: Option<String>) -> ProvenEq {
        let eq = Equation { l: l.clone(), r: r.clone() };
        let proof = Proof::Explicit(j);

        self.prove(eq, proof).unwrap()
    }

    pub fn prove_reflexivity(&self, id: &AppliedId) -> ProvenEq {
        let eq = Equation { l: id.clone(), r: id.clone() };
        let proof = Proof::Reflexivity;

        self.prove(eq, proof).unwrap()
    }

    pub fn prove_symmetry(&self, x: ProvenEq) -> ProvenEq {
        let eq = Equation { l: x.r.clone(), r: x.l.clone() };
        let proof = Proof::Symmetry(x);

        self.prove(eq, proof).unwrap()
    }

    pub fn prove_transitivity(&self, x: ProvenEq, y: ProvenEq) -> ProvenEq {
        let eq = Equation { l: x.l.clone(), r: y.r.clone() };
        let proof = Proof::Transitivity(x.clone(), y.clone());

        let eq1 = x.clone();
        let eq2 = y.clone();
        let theta = match_app_id(&eq2.l, &eq1.r).unwrap();
        let a = eq1.l.clone();
        let c = eq2.r.apply_slotmap_fresh(&theta);
        let out = Equation { l: a, r: c };

        self.prove(eq, proof).unwrap()
    }

    pub fn prove_congruence(&self, l: &AppliedId, r: &AppliedId, child_proofs: Vec<ProvenEq>) -> ProvenEq {
        let eq = Equation { l: l.clone(), r: r.clone() };
        let proof = Proof::Congruence(child_proofs);
        self.prove(eq, proof).unwrap()
    }
}
