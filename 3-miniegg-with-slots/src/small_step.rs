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
            todo!()
        },
        ENode::Lam(y, bb) => {
            todo!()
        },
    }
}
