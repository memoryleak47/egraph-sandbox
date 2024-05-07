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
    match_against(sre, pattern).unwrap().0
}

// Finds a renaming (SlotMap) of the slots of `sre`, so that it becomes equivalent to `pattern`.
// Also extracts the resulting Subst for the Pattern.
// Supports partial `sre`, but obviously it cannot return a Subst-entry for them.
fn match_against<L: Language>(sre: &SemiRecExpr<L>, pattern: &Pattern<L>) -> Option<(Subst, SlotMap)> {
    let mut subst = Subst::default();
    let mut slotmap = SlotMap::new();
    match_against_impl(&sre, pattern, &mut subst, &mut slotmap)?;

    // Previously, the subst uses `sre`-based slot names.
    // Afterwards, the subst uses `pattern`-based slot names.
    for (k, v) in subst.iter_mut() {
        *v = v.apply_slotmap(&slotmap);
    }
    Some((subst, slotmap))
}

// `slotmap` maps from `sre` to `pattern` slots.
// The returned Subst works with `sre` slots.
fn match_against_impl<L: Language>(sre: &SemiRecExpr<L>, pattern: &Pattern<L>, subst: &mut Subst, slotmap: &mut SlotMap) -> Option<()> {
    match (&sre.node, &pattern.node) {
        // the "leaf" case.
        (ENodeOrAppId::AppliedId(x), ENodeOrVar::Var(v)) => {
            try_insert_compatible(v.clone(), x.clone(), subst);
            Some(())
        },

        // the "partial" case.
        (ENodeOrAppId::AppliedId(_), ENodeOrVar::ENode(_)) => {
            Some(())
        },

        // the "equality-check" case.
        (ENodeOrAppId::ENode(n1), ENodeOrVar::ENode(n2)) => {
            let slots1 = n1.all_slot_occurences();
            let slots2 = n2.all_slot_occurences();

            if slots1.len() != slots2.len() { return None; }
            for (&x, &y) in slots1.iter().zip(slots2.iter()) {
                if !try_insert_compatible_slotmap(x, y, slotmap) {
                    return None;
                }
            }
            let check_eq = {
                let mut n1_clone = n1.clone();
                for x in n1_clone.all_slot_occurences_mut() {
                    *x = slotmap[*x];
                }

                n1_clone == *n2
            };
            if !check_eq { return None; }

            for (subsre, subpat) in sre.children.iter().zip(pattern.children.iter()) {
                match_against_impl(subsre, subpat, subst, slotmap)?;
            }

            Some(())
        },

        // the "invalid" case.
        (ENodeOrAppId::ENode(_), ENodeOrVar::Var(_)) => {
            panic!("The sre can never be larger than the pattern!")
        },
    }
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

fn try_insert_compatible_slotmap(k: Slot, v: Slot, map: &mut SlotMap) -> bool {
    if let Some(v_old) = map.get(k) {
        if v_old != v {
            return false;
        }
    }
    map.insert(k, v);
    true
}
