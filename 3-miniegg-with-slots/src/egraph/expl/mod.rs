use crate::*;

mod proof;
pub use proof::*;

mod step;
pub use step::*;

impl<L: Language> EGraph<L> {
    pub fn explain_equivalence(&mut self, t1: RecExpr<L>, t2: RecExpr<L>) -> ProvenEq {
        let i1 = self.add_syn_expr(t1);
        let i2 = self.add_syn_expr(t2);

        if !self.eq(&i1, &i2) { panic!("Can't explain an equivalence that does not hold!"); }

        let prf1 = self.unionfind_get_proof(i1.id);
        let prf2 = self.unionfind_get_proof(i2.id);
        let prf2 = self.prove_symmetry(prf2);
        let p = self.prove_transitivity(prf1, prf2);
        assert_eq!(&p.l, &i1);
        assert_eq!(&p.r, &i2);
        p
    }
}
