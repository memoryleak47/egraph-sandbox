use crate::*;

mod enode_or;
pub use enode_or::*;

mod ematch;
pub use ematch::*;

mod pattern_subst;
pub use pattern_subst::*;

// The AppliedIds in `node` are ignored. They are replaced by the children RecExpr2.
// A non-fancy version of RecExpr that uses the slots as "names".
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RecExpr2<L: Language> {
    pub node: L,
    pub children: Vec<RecExpr2<L>>,
}

pub fn rewrite_if<L: Language>(eg: &mut EGraph<L>, a: Pattern<L>, b: Pattern<L>, cond: impl Fn(&Subst) -> bool) {
    for subst in ematch_all(eg, &a) {
        if cond(&subst) {
            let a = pattern_subst(eg, &a, &subst);
            let b = pattern_subst(eg, &b, &subst);
            eg.union(&a, &b);
        }
    }
}

pub fn rewrite<L: Language>(eg: &mut EGraph<L>, a: Pattern<L>, b: Pattern<L>) {
    rewrite_if(eg, a, b, |_| true);
}
