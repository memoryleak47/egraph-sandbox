use crate::*;

pub fn rewrite_let(eg: &mut EGraph<LetENode>) {
    beta(eg);
    my_let_unused(eg);

    old_propagate_let(eg);
    // let_var_same(eg);

    let_app(eg);
    let_lam_diff(eg);
}

fn old_propagate_let(eg: &mut EGraph<LetENode>) {
    for c in eg.ids() {
        for enode in eg.enodes(c) {
            let id = eg.lookup(&enode).unwrap();
            if let LetENode::Let(x, t, b) = &enode {
                for b2 in eg.enodes_applied(b) {
                    if let Some(new) = old_propagate_let_step(*x, t.clone(), b2, eg) {
                        eg.union(&new, &id);
                    }
                }
            }
        }
    }
}

fn old_propagate_let_step(x: Slot, t: AppliedId, b: LetENode, eg: &mut EGraph<LetENode>) -> Option<AppliedId> {
    // This optimization does soo much for some reason.
    if !b.slots().contains(&x) {
        return Some(eg.lookup(&b).unwrap());
    }

    let out = match b {
        LetENode::Var(_) => t,
        _ => return None,
    };


    Some(out)
}

fn beta(eg: &mut EGraph<LetENode>) {
    // (\s1. ?b) ?t
    let pat = app_pat(lam_pat(Slot::new(1), pvar_pat("?b")), pvar_pat("?t"));

    // let s1 ?t ?b
    let outpat = let_pat(Slot::new(1), pvar_pat("?t"), pvar_pat("?b"));

    rewrite(eg, pat, outpat);
}

fn my_let_unused(eg: &mut EGraph<LetENode>) {
    let pat = let_pat(Slot::new(1), pvar_pat("?t"), pvar_pat("?b"));
    let outpat = pvar_pat("?b");
    rewrite_if(eg, pat, outpat, |subst| {
        !subst["?b"].slots().contains(&Slot::new(1))
    });
}

fn let_var_same(eg: &mut EGraph<LetENode>) {
    let pat = let_pat(Slot::new(1), pvar_pat("?e"), var_pat(Slot::new(1)));
    // let outpat = pvar_pat("?e");
    for subst in ematch_all(eg, &pat) {
        let semi = mk_semi(&pat, &subst);
        let a = add_semi(&semi, eg);
        // let b = pattern_subst(eg, &outpat, &subst);
        // eg.union(&a, &b);
    }
}

fn let_app(eg: &mut EGraph<LetENode>) {
    let pat = let_pat(Slot::new(1), pvar_pat("?e"), app_pat(pvar_pat("?a"), pvar_pat("?b")));
    let outpat = app_pat(
        let_pat(Slot::new(1), pvar_pat("?e"), pvar_pat("?a")),
        let_pat(Slot::new(1), pvar_pat("?e"), pvar_pat("?b"))
    );
    rewrite_if(eg, pat, outpat, |subst| {
        subst["?a"].slots().contains(&Slot::new(1)) || subst["?b"].slots().contains(&Slot::new(1))
    });
}

fn let_lam_diff(eg: &mut EGraph<LetENode>) {
    let pat = let_pat(Slot::new(1), pvar_pat("?e"), lam_pat(Slot::new(2), pvar_pat("?b")));
    let outpat = lam_pat(Slot::new(2),
        let_pat(Slot::new(1), pvar_pat("?e"), pvar_pat("?b")),
    );
    rewrite_if(eg, pat, outpat, |subst| {
        subst["?b"].slots().contains(&Slot::new(1))
    });
}

// aux functions.
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

fn empty_app_id() -> AppliedId { AppliedId::new(Id(0), SlotMap::new()) }
