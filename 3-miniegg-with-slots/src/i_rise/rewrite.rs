use crate::*;

pub fn rewrite_rise(eg: &mut EGraph<RiseENode>) {
    beta(eg);
    eta(eg);

    my_let_unused(eg);
    let_var_same(eg);
    let_app(eg);
    let_lam_diff(eg);

    let_add(eg);

    map_fusion(eg);
    map_fission(eg);
}

fn beta(eg: &mut EGraph<RiseENode>) {
    // (\s1. ?b) ?t
    let pat = app_pat(lam_pat(Slot::new(1), pvar_pat("?b")), pvar_pat("?t"));

    // let s1 ?t ?b
    let outpat = let_pat(Slot::new(1), pvar_pat("?t"), pvar_pat("?b"));

    rewrite(eg, pat, outpat);
}

fn eta(eg: &mut EGraph<RiseENode>) {
    // \s1. ?b s1
    let pat = lam_pat(Slot::new(1), app_pat(pvar_pat("?b"), var_pat(Slot::new(1))));

    // ?b
    let outpat = pvar_pat("?b");

    rewrite_if(eg, pat, outpat, |subst| {
        !subst["?b"].slots().contains(&Slot::new(1))
    });
}

fn my_let_unused(eg: &mut EGraph<RiseENode>) {
    let pat = let_pat(Slot::new(1), pvar_pat("?t"), pvar_pat("?b"));
    let outpat = pvar_pat("?b");
    rewrite_if(eg, pat, outpat, |subst| {
        !subst["?b"].slots().contains(&Slot::new(1))
    });
}

fn let_var_same(eg: &mut EGraph<RiseENode>) {
    let pat = let_pat(Slot::new(1), pvar_pat("?e"), var_pat(Slot::new(1)));
    let outpat = pvar_pat("?e");
    rewrite(eg, pat, outpat);
}

fn let_app(eg: &mut EGraph<RiseENode>) {
    let pat = let_pat(Slot::new(1), pvar_pat("?e"), app_pat(pvar_pat("?a"), pvar_pat("?b")));
    let outpat = app_pat(
        let_pat(Slot::new(1), pvar_pat("?e"), pvar_pat("?a")),
        let_pat(Slot::new(1), pvar_pat("?e"), pvar_pat("?b"))
    );
    rewrite_if(eg, pat, outpat, |subst| {
        subst["?a"].slots().contains(&Slot::new(1)) || subst["?b"].slots().contains(&Slot::new(1))
    });
}

fn let_lam_diff(eg: &mut EGraph<RiseENode>) {
    let pat = let_pat(Slot::new(1), pvar_pat("?e"), lam_pat(Slot::new(2), pvar_pat("?b")));
    let outpat = lam_pat(Slot::new(2),
        let_pat(Slot::new(1), pvar_pat("?e"), pvar_pat("?b")),
    );
    rewrite_if(eg, pat, outpat, |subst| {
        subst["?b"].slots().contains(&Slot::new(1))
    });
}

fn let_add(eg: &mut EGraph<RiseENode>) {
    let pat = let_pat(Slot::new(1), pvar_pat("?e"), add_pat(pvar_pat("?a"), pvar_pat("?b")));
    let outpat = add_pat(
        let_pat(Slot::new(1), pvar_pat("?e"), pvar_pat("?a")),
        let_pat(Slot::new(1), pvar_pat("?e"), pvar_pat("?b"))
    );
    rewrite_if(eg, pat, outpat, |subst| {
        subst["?a"].slots().contains(&Slot::new(1)) || subst["?b"].slots().contains(&Slot::new(1))
    });
}


fn map_fusion(eg: &mut EGraph<RiseENode>) {
    let map = |a, b| app_pat(app_pat(symb_pat("map"), a), b);
    let f = || pvar_pat("?f");
    let g = || pvar_pat("?g");
    let arg = || pvar_pat("?arg");
    let x = Slot::new(0);
    let pat = map(f(),
                map(g(), arg())
              );
    let outpat = map(
            lam_pat(x, app_pat(f(), app_pat(g(), var_pat(x)))),
        arg());
    rewrite(eg, pat, outpat);
}

fn map_fission(eg: &mut EGraph<RiseENode>) {
    let map = |a, b| app_pat(app_pat(symb_pat("map"), a), b);
    let map1 = |a| app_pat(symb_pat("map"), a);
    let f = || pvar_pat("?f");
    let g = || pvar_pat("?g");
    let x = Slot::new(0);
    let y = Slot::new(1);

    let pat = map1(lam_pat(x, app_pat(f(), app_pat(g(), var_pat(x)))));
    let outpat = lam_pat(y, map(f(), map(lam_pat(x, app_pat(g(), var_pat(x))), var_pat(y))));
    rewrite_if(eg, pat, outpat, |subst| {
        !subst["?f"].slots().contains(&x)
    });
}

// aux functions.
fn pvar_pat(s: &str) -> Pattern<RiseENode> {
    Pattern {
        node: ENodeOrPVar::PVar(s.to_string()),
        children: vec![],
    }
}

fn app_pat(l: Pattern<RiseENode>, r: Pattern<RiseENode>) -> Pattern<RiseENode> {
    Pattern {
        node: ENodeOrPVar::ENode(RiseENode::App(empty_app_id(), empty_app_id())),
        children: vec![l, r],
    }
}

fn var_pat(s: Slot) -> Pattern<RiseENode> {
    Pattern {
        node: ENodeOrPVar::ENode(RiseENode::Var(s)),
        children: vec![],
    }
}

fn lam_pat(s: Slot, b: Pattern<RiseENode>) -> Pattern<RiseENode> {
    Pattern {
        node: ENodeOrPVar::ENode(RiseENode::Lam(s, empty_app_id())),
        children: vec![b],
    }
}

fn let_pat(s: Slot, t: Pattern<RiseENode>, b: Pattern<RiseENode>) -> Pattern<RiseENode> {
    Pattern {
        node: ENodeOrPVar::ENode(RiseENode::Let(s, empty_app_id(), empty_app_id())),
        children: vec![t, b],
    }
}

fn empty_app_id() -> AppliedId { AppliedId::new(Id(0), SlotMap::new()) }
