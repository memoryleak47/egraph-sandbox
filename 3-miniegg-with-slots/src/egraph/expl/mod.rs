use crate::*;

mod proof;
pub use proof::*;

mod step;
pub use step::*;

impl<L: Language> EGraph<L> {
    pub fn explain_equivalence(&mut self, t1: RecExpr<L>, t2: RecExpr<L>) -> ProvenEq {
        let i1 = self.add_syn_expr(t1);
        let i2 = self.add_syn_expr(t2);
        todo!() // TODO
        // let p1 = self.unionfind[i1].pointer_to_leader;
        // let p2 = self.unionfind[i2].pointer_to_leader;
        // let (p1, p2) = self.group_permute_to_match(p1, p2);
        // return p1.chain(p2);
    }
}
