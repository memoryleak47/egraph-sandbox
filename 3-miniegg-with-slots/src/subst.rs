use crate::*;

// returns an eclass containing b[x := t].
// All branches that require alpha-conversion will be dropped. Hence, this is incomplete.
pub fn subst(b: Id, x: &str, t: Id, eg: &mut EGraph) -> Id {
    subst_impl(b, x, t, eg, &mut Default::default())
}

// `map` caches the b -> b[x := t] mapping.
fn subst_impl(b: Id, x: &str, t: Id, eg: &mut EGraph, map: &mut HashMap<Id, Id>) -> Id {
    if let Some(o) = map.get(&b) {
        return *o;
    }

    let new_b = eg.alloc_eclass();
    map.insert(b, new_b);

    let b_enodes: Vec<_> = eg.enodes(b).iter().cloned().collect();

    for enode in b_enodes {
        // if the enode_subst failed: we don't add anything!
        if let Some(id) = enode_subst(enode, b, x, t, eg, map) {
            eg.union(new_b, id);
        }
    }

    new_b
}

// `enode` is an enode from the eclass `b`.
// we return an eclass containing `enode[x := t]`
fn enode_subst(enode: ENode, b: Id, x: &str, t: Id, eg: &mut EGraph, map: &mut HashMap<Id, Id>) -> Option<Id> {
    match enode {
        // (lam x2 b2)[x := t] --> (lam x2 b2), if x = x2.
        // In other words, we don't change anything, if x gets re-bound.
        ENode::Lam(x2, _) if x2 == x => Some(b),

        // (lam x2 b2)[x := t] --> (lam x2 b2[x := t]), if x != x2.
        ENode::Lam(x2, b2) => {
            if comes_up_free(&x2, t, eg) {
                // problem: x2 might be free in t.
                // return None in that case!
                return None;
            }

            let b2 = subst_impl(b2, x, t, eg, map);
            Some(eg.add(ENode::Lam(x2, b2)))
        }

        // (app l r)[x := t] --> (app l[x := t] r[x := t])
        ENode::App(l, r) => {
            let l = subst_impl(l, x, t, eg, map);
            let r = subst_impl(r, x, t, eg, map);
            Some(eg.add(ENode::App(l, r)))
        },

        // x2[x := t] --> t, if x = x2.
        ENode::Var(x2) if x == x2 => Some(t),

        // x2[x := t] --> x2, if x != x2.
        ENode::Var(_) => Some(b),
    }
}


// returns whether x in free(t).
fn comes_up_free(x: &str, t: Id, eg: &EGraph) -> bool {
    comes_up_free_impl(x, t, eg, &mut HashSet::new())
}

// `set` stores whether an eclass `t` was already checked.
fn comes_up_free_impl(x: &str, t: Id, eg: &EGraph, set: &mut HashSet<Id>) -> bool {
    if set.contains(&t) { return false; }
    set.insert(t);

    let t_enodes: Vec<_> = eg.enodes(t).iter().cloned().collect();

    for enode in t_enodes {
        match enode {
            ENode::Lam(x2, b) => {
                if x != x2 {
                    if comes_up_free_impl(x, b, eg, set) { return true; }
                }
            }
            ENode::App(l, r) => {
                if comes_up_free_impl(x, l, eg, set) { return true; }
                if comes_up_free_impl(x, r, eg, set) { return true; }
            },
            ENode::Var(x2) => {
                if x == x2 {
                    return true;
                }
            },
        }
    }

    false
}

