use crate::*;

mod proof;
pub use proof::*;

impl<L: Language> EGraph<L> {
    pub fn explain_equivalence(&mut self, t1: RecExpr<L>, t2: RecExpr<L>) -> ProvenEq {
        let i1 = self.add_synt_expr(t1);
        let i2 = self.add_synt_expr(t2);
        let p1 = self.explain_lookup(&i1);
        let p2 = self.explain_lookup(&i2);
        todo!() // chain p1 with p2 and return.
    }

    // proofs that synt_term(i) and synt_term(lookup(i)) are semantically equal.
    fn explain_lookup(&self, i: &AppliedId) -> ProvenEq {
        todo!()
    }
}
