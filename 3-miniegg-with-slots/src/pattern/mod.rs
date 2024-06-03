use crate::*;

mod enode_or;
pub use enode_or::*;

mod ematch;
pub use ematch::*;

mod pattern_subst;
pub use pattern_subst::*;


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

pub fn rewrite_bi<L: Language>(eg: &mut EGraph<L>, a: Pattern<L>, b: Pattern<L>) {
    rewrite(eg, a.clone(), b.clone());
    rewrite(eg, b, a);
}
