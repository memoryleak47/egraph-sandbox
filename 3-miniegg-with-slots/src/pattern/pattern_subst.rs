use crate::*;

// We write this as pattern[subst] for short.
pub fn pattern_subst<L: Language>(eg: &mut EGraph<L>, pattern: &Pattern<L>, subst: &Subst) -> AppliedId {
    match &pattern.node {
        ENodeOrVar::ENode(n) => {
            let mut n = n.clone();
            let mut refs: Vec<&mut _> = n.applied_id_occurences_mut();
            assert_eq!(pattern.children.len(), refs.len());
            for i in 0..refs.len() {
                *(refs[i]) = pattern_subst(eg, &pattern.children[i], subst);
            }
            eg.add(n)
        },
        ENodeOrVar::Var(v) => {
            subst[v].clone()
        },
    }
}

// TODO do I want these?
fn add_semi<L: Language>(semi: SemiRecExpr<L>, eg: &mut EGraph<L>) -> AppliedId {
    todo!()
}

fn mk_semi<L: Language>(pattern: &Pattern<L>, subst: &Subst) -> SemiRecExpr<L> {
    todo!()
}
