use crate::*;

pub fn rewrite_let(eg: &mut EGraph<LetENode>) {
    beta_to_let(eg);
    propagate_let(eg);
}


fn beta_to_let(eg: &mut EGraph<LetENode>) {
    // (\s1. ?b) ?t
    let pat = app_pat(lam_pat(Slot(1), pvar_pat("?b")), pvar_pat("?t"));

    // let s1 ?t ?b
    let outpat = let_pat(Slot(1), pvar_pat("?t"), pvar_pat("?b"));

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


// aux functions.
fn empty_app_id() -> AppliedId { AppliedId::new(Id(0), SlotMap::new()) }
fn pvar_pat(s: &str) -> Pattern<LetENode> {
    Pattern {
        node: ENodeOrPVar::PVar(s.to_string()),
        children: vec![],
    }
}

fn app_pat(l: Pattern<LetENode>, r: Pattern<LetENode>) -> Pattern<LetENode> {
    Pattern {
        node: ENodeOrPVar::ENode(LetENode::App(empty_app_id(), empty_app_id())),
        children: vec![l, r],
    }
}

fn var_pat(s: Slot) -> Pattern<LetENode> {
    Pattern {
        node: ENodeOrPVar::ENode(LetENode::Var(s)),
        children: vec![],
    }
}

fn lam_pat(s: Slot, b: Pattern<LetENode>) -> Pattern<LetENode> {
    Pattern {
        node: ENodeOrPVar::ENode(LetENode::Lam(s, empty_app_id())),
        children: vec![b],
    }
}

fn let_pat(s: Slot, t: Pattern<LetENode>, b: Pattern<LetENode>) -> Pattern<LetENode> {
    Pattern {
        node: ENodeOrPVar::ENode(LetENode::Let(s, empty_app_id(), empty_app_id())),
        children: vec![t, b],
    }
}
