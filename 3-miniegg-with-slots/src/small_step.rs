use crate::*;

pub fn rewrite_small_step(eg: &mut EGraph) {
    for cand in candidates(eg) {
        let app_id = eg.lookup(&cand.app).unwrap();

        // L0 = ENode::App(l, t).slots() -- "the root level"
        // t.slots(), l.slots(), app_id.slots() :: L0

        // L1 = ENode::Lam(x, b).slots() = slots(l.id)

        let ENode::App(l, t) = cand.app.clone() else { panic!() };
        let ENode::Lam(x, b) = cand.lam.clone() else { panic!() };
        assert_eq!(x, Slot(0));

        // b.m :: slots(b.id) -> L1
        // l.m :: slots(l.id) -> L0 (and thus L1 -> L0)

        // The L0-equivalent of x.
        let x_root = Slot::fresh();

        let mut l_m = l.m.clone();
        l_m.insert(x, x_root);
        let b = b.apply_slotmap(&l_m);

        for b_node in eg.enodes_applied(&b) {
            let new = step(x_root, t.clone(), &b_node, eg);
            eg.union(new, app_id.clone());
        }
    }
}

// everything here has L0 slot-names.
fn step(x: Slot, t: AppliedId, b: &ENode, eg: &mut EGraph) -> AppliedId {
    if !b.slots().contains(&x) {
        return eg.lookup(b).unwrap();
    }

    match b {
        ENode::Var(_) => t,
        ENode::App(l, r) => {
            let mut pack = |lr: &AppliedId| {
                let a1 = eg.add(ENode::Lam(x, lr.clone()));
                let a2 = eg.add(ENode::App(a1, t.clone()));
                a2
            };
            let l = pack(l);
            let r = pack(r);
            eg.add(ENode::App(l, r))
        },
        ENode::Lam(y, bb) => {
            let a1 = eg.add(ENode::Lam(x, bb.clone()));
            let a2 = eg.add(ENode::App(a1, t.clone()));
            let a3 = eg.add(ENode::Lam(*y, a2));
            a3
        },
    }
}
