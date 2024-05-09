use crate::*;

// We write this as pattern[subst] for short.
pub fn pattern_subst<L: Language>(eg: &mut EGraph<L>, pattern: &Pattern<L>, subst: &Subst) -> AppliedId {
    match &pattern.node {
        ENodeOrPVar::ENode(n) => {
            let mut n = n.clone();
            let mut refs: Vec<&mut _> = n.applied_id_occurences_mut();
            assert_eq!(pattern.children.len(), refs.len());
            for i in 0..refs.len() {
                *(refs[i]) = pattern_subst(eg, &pattern.children[i], subst);
            }
            eg.add(n)
        },
        ENodeOrPVar::PVar(v) => {
            subst[v].clone()
        },
    }
}

// Should be equivalent to pattern_subst.
pub fn pattern_subst2<L: Language>(eg: &mut EGraph<L>, pattern: &Pattern<L>, subst: &Subst) -> AppliedId {
    let semi = mk_semi(pattern, subst);
    let app_id = add_semi(&semi, eg);
    app_id
}

fn mk_semi<L: Language>(pattern: &Pattern<L>, subst: &Subst) -> SemiRecExpr<L> {
    match &pattern.node {
        ENodeOrPVar::ENode(n) => {
            let children: Vec<SemiRecExpr<L>> = pattern.children.iter().map(|x| mk_semi(x, subst)).collect();
            SemiRecExpr {
                node: ENodeOrAppId::ENode(n.clone()),
                children,
            }
        },
        ENodeOrPVar::PVar(v) => {
            SemiRecExpr {
                node: ENodeOrAppId::AppliedId(subst[v].clone()),
                children: vec![],
            }
        },
    }
}

fn add_semi<L: Language>(semi: &SemiRecExpr<L>, eg: &mut EGraph<L>) -> AppliedId {
    match &semi.node {
        ENodeOrAppId::ENode(n) => {
            let mut n = n.clone();
            let mut refs: Vec<&mut AppliedId> = n.applied_id_occurences_mut();
            assert_eq!(semi.children.len(), refs.len());
            for i in 0..refs.len() {
                *(refs[i]) = add_semi(&semi.children[i], eg);
            }
            eg.add(n)
        },
        ENodeOrAppId::AppliedId(app_id) => app_id.clone(),
    }
}
