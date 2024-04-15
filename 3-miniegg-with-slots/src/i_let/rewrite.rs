use crate::*;

pub fn rewrite_let(eg: &mut EGraph<LetENode>) {
    beta_to_let(eg);
    propagate_let(eg);
}

fn beta_to_let(eg: &mut EGraph<LetENode>) {
    for cand in beta_candidates(eg) {
        let app_id = eg.lookup(&cand.app).unwrap();

        // L0 = ENode::App(l, t).slots() -- "the root level"
        // t.slots(), l.slots(), app_id.slots() :: L0

        // L1 = ENode::Lam(x, b).slots() = slots(l.id)

        let LetENode::App(l, t) = cand.app.clone() else { panic!() };
        let LetENode::Lam(x, b) = cand.lam.clone() else { panic!() };
        assert_eq!(x, Slot(0));

        // b.m :: slots(b.id) -> L1
        // l.m :: slots(l.id) -> L0 (and thus L1 -> L0)

        // The L0-equivalent of x.
        let x_root = Slot::fresh();

        let mut l_m = l.m.clone();
        l_m.insert(x, x_root);
        let b = b.apply_slotmap(&l_m);

        let new = LetENode::Let(x_root, t, b);
        let new = eg.add(new);
        eg.union(new, app_id.clone());
    }
}

fn propagate_let(eg: &mut EGraph<LetENode>) {
    for c in eg.ids() {
        for enode in eg.enodes(c) {
            let id = eg.lookup(&enode).unwrap();
            if let LetENode::Let(x, t, b) = &enode {
                for b2 in eg.enodes_applied(b) {
                    if let Some(new) = propagate_let_step(*x, t.clone(), b2, eg) {
                        eg.union(new, id.clone());
                    }
                }
            }
        }
    }
}

fn propagate_let_step(x: Slot, t: AppliedId, b: LetENode, eg: &mut EGraph<LetENode>) -> Option<AppliedId> {
    // TODO re-enable optimization:

    // if !b.slots().contains(&x) {
    //    return Some(eg.lookup(&b).unwrap());
    // }

    let out = match b {
        LetENode::Var(_) => {
            if b.slots().contains(&x) {
                t
            } else {
                eg.lookup(&b).unwrap()
            }
        },
        LetENode::App(l, r) => {
            let l = eg.add(LetENode::Let(x, t.clone(), l));
            let r = eg.add(LetENode::Let(x, t.clone(), r));
            eg.add(LetENode::App(l, r))
        },
        LetENode::Lam(y, bb) => {
            let a1 = eg.add(LetENode::Let(x, t, bb.clone()));
            let a2 = eg.add(LetENode::Lam(y, a1));
            a2
        },
        LetENode::Let(..) => return None,
    };

    Some(out)
}

// candidate for beta reduction.
// Both ENodes are computed by "sh.apply_slotmap(bij)", where (sh, bij) in EClass::nodes from their respective classes.
struct BetaCandidate {
    pub app: LetENode,
    pub lam: LetENode,
}

fn beta_candidates(eg: &EGraph<LetENode>) -> Vec<BetaCandidate> {
    // find all lambdas:
    let mut lambdas: HashMap<Id, Vec<LetENode>> = Default::default();
    for c in eg.ids() {
        let mut v = Vec::new();
        for enode in eg.enodes(c) {
            if matches!(enode, LetENode::Lam(..)) {
                v.push(enode.clone());
            }
        }

        lambdas.insert(c, v);
    }

    // find apps:
    let mut candidates = Vec::new();

    for c in eg.ids() {
        for enode in eg.enodes(c) {
            if let LetENode::App(l, _t) = &enode {
                for lam in lambdas[&l.id].clone() {
                    candidates.push(BetaCandidate { app: enode.clone(), lam });
                }
            }
        }
    }

    candidates
}



