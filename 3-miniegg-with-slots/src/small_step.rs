use crate::*;

pub fn rewrite_small_step(eg: &mut EGraph) {
    for cand in candidates(eg) {
        let app_id = eg.lookup(&cand.app).unwrap();

        let ENode::App(l, t) = cand.app.clone() else { panic!() };
        let ENode::Lam(x, b) = cand.lam.clone() else { panic!() };
        assert_eq!(x, Slot(0));

        for b in &eg.enodes(b.id) {
            let new = step(x, t.clone(), b, eg);
            eg.union(new, app_id.clone());
        }
    }
}

// TODO
fn step(x: Slot, t: AppliedId, b: &ENode, eg: &mut EGraph) -> AppliedId {
    match b {
        ENode::Var(y) => {
            if *y == x {
                t
            } else {
                eg.add(ENode::Var(*y))
            }
        },
        ENode::App(l, r) => {
            let mut pack = |b: &AppliedId| {
                let a = eg.add(ENode::Lam(x, b.clone()));
                let out = eg.add(ENode::App(a, t.clone()));
                out
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
