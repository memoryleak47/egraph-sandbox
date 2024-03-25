use crate::*;

pub fn beta_reduction() -> Rewrite<ENode, Varbound> {
    rewrite!("beta-reduction"; "(app (lam ?b) ?t)" => { BetaReduction })
}

struct BetaReduction;

impl Applier<ENode, Varbound> for BetaReduction {
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
    let ctxt = &mut Ctxt::default();

    let out = beta_subst_impl(b, 0, t, eg, ctxt);

    for (x, y) in &ctxt.future_unions {
        eg.union(*x, *y);
    }

    out
}

#[derive(Default)]
struct Ctxt {
    // (b, x) -> b[x := t]
    subst_map: HashMap<(Id, u32), Id>,

    // (t, offset, min_free) -> t[i := i+offset, i >= min_free]
    shift_map: HashMap<(Id, u32, u32), Id>,

    // Unions whose execution is deferred to the end of this algorithm.
    future_unions: Vec<(Id, Id)>,
}

fn beta_subst_impl(b: Id, x: u32, t: Id, eg: &mut EG, ctxt: &mut Ctxt) -> Id {
    if x > eg[b].data+1 { // TODO re-consider this check.
        return b;
    }

    if let Some(out) = ctxt.subst_map.get(&(b, x)) {
        return *out;
    }

    let new = alloc_eclass(eg);
    ctxt.subst_map.insert((b, x), new);

    for enode in eg[b].nodes.clone() {
        if matches!(enode, ENode::Placeholder(_)) { continue; }

        let elem = beta_subst_enode(enode, x, t, eg, ctxt);
        ctxt.future_unions.push((new, elem));
    }

    new
}

fn beta_subst_enode(b: ENode, x: u32, t: Id, eg: &mut EG, ctxt: &mut Ctxt) -> Id {
    match b {
        ENode::Lam(b) => {
            let b = beta_subst_impl(b, x+1, t, eg, ctxt);
            eg.add(ENode::Lam(b))
        },
        ENode::App([l, r]) => {
            let l = beta_subst_impl(l, x, t, eg, ctxt);
            let r = beta_subst_impl(r, x, t, eg, ctxt);

            eg.add(ENode::App([l, r]))
        },
        ENode::Var(i) => {
            if i == x {
                shift(t, x, eg, ctxt)
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

// shifts all free(!) variables in t by "offset".
fn shift(t: Id, offset: u32, eg: &mut EG, ctxt: &mut Ctxt) -> Id {
    shift_impl(t, offset, 0, eg, ctxt)
}

fn shift_impl(t: Id, offset: u32, min_free: u32, eg: &mut EG, ctxt: &mut Ctxt) -> Id {
    if min_free > eg[t].data+1 { // TODO re-consider this check.
        return t;
    }

    if let Some(out) = ctxt.shift_map.get(&(t, offset, min_free)) {
        return *out;
    }

    let out = alloc_eclass(eg);
    ctxt.shift_map.insert((t, offset, min_free), out);

    for enode in eg[t].nodes.clone() {
        if matches!(enode, ENode::Placeholder(_)) { continue; }

        let elem = shift_enode(enode, offset, min_free, eg, ctxt);
        ctxt.future_unions.push((elem, out));
    }

    out
}

fn shift_enode(t: ENode, offset: u32, min_free: u32, eg: &mut EG, ctxt: &mut Ctxt) -> Id {
    match t {
        ENode::App([l, r]) => {
            let l = shift_impl(l, offset, min_free, eg, ctxt);
            let r = shift_impl(r, offset, min_free, eg, ctxt);
            eg.add(ENode::App([l, r]))
        },
        ENode::Lam(b) => {
            let b = shift_impl(b, offset, min_free+1, eg, ctxt);
            eg.add(ENode::Lam(b))
        },
        ENode::Var(i) => {
            let real_off = if i >= min_free { offset } else { 0 };
            eg.add(ENode::Var(i+real_off))
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
