use crate::*;

impl<L: Language> EGraph<L> {
    pub fn prove_explicit(&self, eq: Equation, j: Option<String>) -> Arc<ProvenEq> {
        let proof = Proof::Explicit(j);

        self.prove(eq, proof).unwrap()
    }

    pub fn prove_reflexivity(&self, id: &AppliedId) -> Arc<ProvenEq> {
        let eq = Equation { l: id.clone(), r: id.clone() };
        let proof = Proof::Reflexivity;

        self.prove(eq, proof).unwrap()
    }

    pub fn prove_symmetry(&self, x: Arc<ProvenEq>) -> Arc<ProvenEq> {
        let eq = Equation { l: x.eq().r.clone(), r: x.eq().l.clone() };
        let proof = Proof::Symmetry(x);

        self.prove(eq, proof).unwrap()
    }

    pub fn prove_transitivity(&self, x: Arc<ProvenEq>, y: Arc<ProvenEq>) -> Arc<ProvenEq> {
        let eq = Equation { l: x.eq().r.clone(), r: x.eq().l.clone() };
        let proof = Proof::Transitivity(x.clone(), y.clone());

        let eq1 = x.eq().clone();
        let eq2 = y.eq().clone();
        let theta = match_app_id(&eq2.l, &eq1.r).unwrap();
        let a = eq1.l.clone();
        let c = eq2.r.apply_slotmap_fresh(&theta);
        let out = Equation { l: a, r: c };

        self.prove(eq, proof).unwrap()
    }
}
