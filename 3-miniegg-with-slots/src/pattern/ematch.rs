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
                    node: ENodeOrAppId::ENode(clear_app_ids(&l)),
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

fn clear_app_ids<L: Language>(l: &L) -> L {
    let mut l = l.clone();
    for x in l.applied_id_occurences_mut() {
        *x = AppliedId::new(Id(0), SlotMap::new());
    }
    l
}

fn compatible<L: Language>(sre: &SemiRecExpr<L>, pattern: &Pattern<L>) -> bool {
    match_against(sre, pattern).is_some()
}

fn to_subst<L: Language>(sre: &SemiRecExpr<L>, pattern: &Pattern<L>) -> Subst {
    match_against(sre, pattern).unwrap()
}

// Returns None if there is an inconsistency between `sre` and `pattern`.
// `sre` is allowed to be partial though!
fn match_against<L: Language>(sre: &SemiRecExpr<L>, pattern: &Pattern<L>) -> Option<Subst> {
    let Some(sre) = rename_to_fit(sre, pattern) else { return None };
    match_against_impl(&sre, pattern)
}

fn match_against_impl<L: Language>(sre: &SemiRecExpr<L>, pattern: &Pattern<L>) -> Option<Subst> {
    let mut subst = Subst::default();

    match (&sre.node, &pattern.node) {
        // the "leaf" case.
        (ENodeOrAppId::AppliedId(x), ENodeOrVar::Var(v)) => {
            subst.insert(v.clone(), x.clone());
            Some(subst)
        },

        // the "partial" case.
        (ENodeOrAppId::AppliedId(_), ENodeOrVar::ENode(_)) => {
            Some(subst)
        },

        // the "equality-check" case.
        (ENodeOrAppId::ENode(n1), ENodeOrVar::ENode(n2)) => {
            if n1 != n2 {
                return None;
            }
            for (subsre, subpat) in sre.children.iter().zip(pattern.children.iter()) {
                for (x, y) in match_against_impl(subsre, subpat)? {
                    if !try_insert_compatible(x, y, &mut subst) { return None; }
                }
            }

            Some(subst)
        },

        // the "invalid" case.
        (ENodeOrAppId::ENode(_), ENodeOrVar::Var(_)) => {
            panic!("The sre can never be larger than the pattern!")
        },
    }
}

// Renames all Slots in `sre` to match `pattern`. The slots that don't come up in `pattern` are still arbitrarily named.
// Returns None if `sre` and `pattern` are conceptually different.
// Also supports partial `sre`s that don't cover all of `pattern`.
fn rename_to_fit<L: Language>(sre: &SemiRecExpr<L>, pattern: &Pattern<L>) -> Option<SemiRecExpr<L>> {
    todo!()
}

fn try_insert_compatible<K: Hash + Eq, V: Eq>(k: K, v: V, map: &mut HashMap<K, V>) -> bool {
    if let Some(v_old) = map.get(&k) {
        if v_old != &v {
            return false;
        }
    }
    map.insert(k, v);
    true
}
