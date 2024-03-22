use crate::*;

pub fn beta_reduction() -> Rewrite<ENode, ()> {
    rewrite!("beta-reduction"; "(app (lam ?b) ?t)" => { BetaReduction })
}

struct BetaReduction;

impl Applier<ENode, ()> for BetaReduction {
    fn apply_one(&self, eg: &mut EG, id: Id, subst: &Subst, _pat: Option<&PatternAst<ENode>>, _rule_name: Symbol) -> Vec<Id> {
        let b: Var = "?b".parse().unwrap();
        let t: Var = "?t".parse().unwrap();

        let b: Id = subst[b];
        let t: Id = subst[t];

        let new = beta_substitution(b, t, eg);
        eg.union(new, id);

        Vec::new() // is this fine?
    }
}

fn beta_substitution(b: Id, t: Id, eg: &mut EG) -> Id {
    beta_subst_impl(b, 0, t, eg, &mut Default::default(), &mut Default::default())
}

// subst_map[b, x] = b[x := t]
// shift_map[b, x] = b[shifted by x]
fn beta_subst_impl(b: Id, x: u32, t: Id, eg: &mut EG, subst_map: &mut HashMap<(Id, u32), Id>, shift_map: &mut HashMap<(Id, u32), Id>) -> Id {
    if let Some(out) = subst_map.get(&(b, x)) {
        return *out;
    }

    let new = alloc_eclass(eg);

    for enode in eg[b].nodes.clone() {
        if matches!(enode, ENode::Placeholder(_)) { continue; }

        let elem = beta_subst_enode(enode, x, t, eg, subst_map, shift_map);
        eg.union(new, elem);
    }

    new
}

fn beta_subst_enode(b: ENode, x: u32, t: Id, eg: &mut EG, subst_map: &mut HashMap<(Id, u32), Id>, shift_map: &mut HashMap<(Id, u32), Id>) -> Id {
    match b {
        ENode::Lam(b) => {
            let b = beta_subst_impl(b, x+1, t, eg, subst_map, shift_map);
            eg.add(ENode::Lam(b))
        },
        ENode::App([l, r]) => {
            let l = beta_subst_impl(l, x, t, eg, subst_map, shift_map);
            let r = beta_subst_impl(r, x, t, eg, subst_map, shift_map);

            eg.add(ENode::App([l, r]))
        },
        ENode::Var(i) => {
            if i == x {
                shift(x, t, eg, shift_map)
            } else if i < x {
                // It's a "local" reference. Keep it unchanged.
                eg.add(ENode::Var(i))
            } else {
                // i > x.
                // It's a "global" reference.
                // We are losing a layer of "lam" due to the beta reduction.
                eg.add(ENode::Var(i-1))
            }
        },
        ENode::Placeholder(_) => panic!("subst_enode: Cannot substitute in a placeholder!"),
    }
}

// shifts all variables in t by x.
fn shift(x: u32, t: Id, eg: &mut EG, shift_map: &mut HashMap<(Id, u32), Id>) -> Id {
    if let Some(out) = shift_map.get(&(t, x)) {
        return *out;
    }

    let out = alloc_eclass(eg);

    for enode in eg[t].nodes.clone() {
        if matches!(enode, ENode::Placeholder(_)) { continue; }

        let elem = shift_enode(x, enode, eg, shift_map);
        eg.union(elem, out);
    }

    out
}

fn shift_enode(x: u32, t: ENode, eg: &mut EG, shift_map: &mut HashMap<(Id, u32), Id>) -> Id {
    match t {
        ENode::App([l, r]) => {
            let l = shift(x, l, eg, shift_map);
            let r = shift(x, r, eg, shift_map);
            eg.add(ENode::App([l, r]))
        },
        ENode::Lam(b) => {
            let b = shift(x, b, eg, shift_map);
            eg.add(ENode::Lam(b))
        },
        ENode::Var(i) => {
            eg.add(ENode::Var(i+x))
        },
        ENode::Placeholder(_) => panic!(),
    }
}

// allocates a new (conceptually empty) eclass.
fn alloc_eclass(eg: &mut EG) -> Id {
    use std::sync::atomic::*;

    static GLOBAL_CTR: AtomicUsize = AtomicUsize::new(0);
    let num = GLOBAL_CTR.fetch_add(1, Ordering::SeqCst);

    let num = eg.add(ENode::Var(num as u32));
    let num = eg.add(ENode::Placeholder(num));

    num
}
