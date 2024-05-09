use crate::*;

pub fn rewrite_let(eg: &mut EGraph<LetENode>) {
    beta_to_let(eg);
    propagate_let(eg);
}

fn beta_to_let(eg: &mut EGraph<LetENode>) {
    // (\s1. ?b) ?t
    let empty = || AppliedId::new(Id(0), SlotMap::new());
    let var = |s: &str| Pattern {
        node: ENodeOrVar::Var(s.to_string()),
        children: vec![],
    };
    let lam = Pattern {
        node: ENodeOrVar::ENode(LetENode::Lam(Slot(1), empty())),
        children: vec![var("?b")],
    };
    let pat = Pattern {
        node: ENodeOrVar::ENode(LetENode::App(empty(), empty())),
        children: vec![lam, var("?t")],
    };

    // let s1 ?t ?b
    let outpat = Pattern {
        node: ENodeOrVar::ENode(LetENode::Let(Slot(1), empty(), empty())),
        children: vec![var("?t"), var("?b")],
    };

    for subst in ematch_all(eg, &pat) {
        let a = pattern_subst(eg, &pat, &subst);
        let b = pattern_subst(eg, &outpat, &subst);
        eg.union(&a, &b);
    }
}

fn propagate_let(eg: &mut EGraph<LetENode>) {
    for c in eg.ids() {
        for enode in eg.enodes(c) {
            let id = eg.lookup(&enode).unwrap();
            if let LetENode::Let(x, t, b) = &enode {
                for b2 in eg.enodes_applied(b) {
                    if let Some(new) = propagate_let_step(*x, t.clone(), b2, eg) {
                        eg.union(&new, &id);
                    }
                }
            }
        }
    }
}

fn propagate_let_step(x: Slot, t: AppliedId, b: LetENode, eg: &mut EGraph<LetENode>) -> Option<AppliedId> {
    // This optimization does soo much for some reason.
    if !b.slots().contains(&x) {
        return Some(eg.lookup(&b).unwrap());
    }

    let out = match b {
        LetENode::Var(_) => t,
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
