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
    let mut ctxt = &mut Ctxt::default();
    let out = beta_subst_impl(b, 0, t, eg, ctxt);

    for (x, y) in &ctxt.future_unions {
        eg.union(*x, *y);
    }

    out
}

#[derive(Default)]
struct Ctxt {
    subst_map: HashMap<(Id, u32), Id>,
    shift_map: HashMap<(Id, u32), Id>,
    // TODO add: max_var_map: HashMap<Id, u32>,
    future_unions: Vec<(Id, Id)>,
}

// subst_map[b, x] = b[x := t]
// shift_map[b, x] = b[shifted by x]
fn beta_subst_impl(b: Id, x: u32, t: Id, eg: &mut EG, ctxt: &mut Ctxt) -> Id {
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
                shift(x, t, eg, ctxt)
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
fn shift(x: u32, t: Id, eg: &mut EG, ctxt: &mut Ctxt) -> Id {
    if let Some(out) = ctxt.shift_map.get(&(t, x)) {
        return *out;
    }

    let out = alloc_eclass(eg);
    ctxt.shift_map.insert((t, x), out);

    for enode in eg[t].nodes.clone() {
        if matches!(enode, ENode::Placeholder(_)) { continue; }

        let elem = shift_enode(x, enode, eg, ctxt);
        ctxt.future_unions.push((elem, out));
    }

    out
}

fn shift_enode(x: u32, t: ENode, eg: &mut EG, ctxt: &mut Ctxt) -> Id {
    match t {
        ENode::App([l, r]) => {
            let l = shift(x, l, eg, ctxt);
            let r = shift(x, r, eg, ctxt);
            eg.add(ENode::App([l, r]))
        },
        ENode::Lam(b) => {
            let b = shift(x, b, eg, ctxt);
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
