use crate::*;

impl<L: Language> EGraph<L> {
    pub fn prove_symmetry(&self, x: Arc<ProvenEq>) -> Arc<ProvenEq> {
        let eq = Equation { l: x.eq().r.clone(), r: x.eq().l.clone() };
        let proof = Proof::Symmetry(x);

        self.prove(eq, proof).unwrap()
    }
}
