use crate::*;

pub type Subst = HashMap<String, AppliedId>;

pub fn ematch<L: Language>(eg: &EGraph<L>, pattern: &Pattern<L>) -> Vec<Subst> {
    let mut out = Vec::new();
    for i in eg.ids() {
        // invariant: each x in worklist satisfies compatible(x, pattern)
        let mut worklist = vec![leaf(eg.mk_identity_applied_id(i))];
        while let Some(x) = worklist.pop() {
            if let Some(xs) = branch(&x, pattern, eg) {
                for y in xs {
                    if compatible(&y, pattern) {
                        worklist.push(y);
                    }
                }
            } else {
                out.push(to_subst(&x, pattern));
            }
        }
    }
    out
}

fn leaf<L: Language>(x: AppliedId) -> SemiRecExpr<L> {
    let x = ENodeOrAppId::AppliedId(x);
    let x = SemiRecExpr { node: x, children: vec![] };
    x
}

// If the SemiRecExpr already covers the whole pattern, we return None.
// Otherwise, we extend the SemiRecExpr at some point and return all possible e-node extensions for that spot.
fn branch<L: Language>(sre: &SemiRecExpr<L>, pattern: &Pattern<L>, eg: &EGraph<L>) -> Option<Vec<SemiRecExpr<L>>> {
    match (&sre.node, &pattern.node) {
        // Here we can extend the SemiRecExpr:
        (ENodeOrAppId::AppliedId(id), ENodeOrVar::ENode(n)) => {
            let mut out = Vec::new();
            for l in eg.enodes_applied(id) {
                let new_sre = SemiRecExpr {
                    node: ENodeOrAppId::ENode(l.clone()),
                    children: l.applied_id_occurences().into_iter().map(leaf).collect(),
                };
                out.push(new_sre);
            }
            Some(out)
        },
        (ENodeOrAppId::ENode(_), ENodeOrVar::ENode(_)) => {
            assert_eq!(sre.children.len(), pattern.children.len());
            for i in 0..sre.children.len() {
                let subsre = &sre.children[i];
                let subpat = &pattern.children[i];
                if let Some(subs) = branch(subsre, subpat, eg) {
                    let mut out = Vec::new();
                    for sub in subs {
                        let mut option = sre.clone();
                        option.children[i] = sub;
                        out.push(option);
                    }
                    return Some(out);
                }
            }
            None
        },
        (ENodeOrAppId::AppliedId(_), ENodeOrVar::Var(_)) => None,
        (ENodeOrAppId::ENode(_), ENodeOrVar::Var(_)) => panic!(),
    }
}

fn compatible<L: Language>(sre: &SemiRecExpr<L>, pattern: &Pattern<L>) -> bool {
    todo!()
}

fn to_subst<L: Language>(sre: &SemiRecExpr<L>, pattern: &Pattern<L>) -> Subst {
    todo!()
}
