use miniegg_with_slots::*;

fn main() {
    let mut eg: &mut EGraph<RiseENode> = &mut EGraph::new();
    let id = |s, eg: &mut EGraph<RiseENode>| -> AppliedId {
        let re = RecExpr::parse(s).unwrap();
        eg.add_syn_expr(re.clone())
    };

    let term = |s, eg: &mut EGraph<RiseENode>| -> RecExpr<RiseENode> {
        let re = RecExpr::parse(s).unwrap();
        eg.add_syn_expr(re.clone());
        re
    };

    let equate = |s1, s2, eg: &mut EGraph<RiseENode>| {
        let s1 = id(s1, eg);
        let s2 = id(s2, eg);
        eg.union(&s1, &s2);
    };

    let explain = |s1, s2, eg: &mut EGraph<RiseENode>| {
        let s1 = term(s1, eg);
        let s2 = term(s2, eg);
        eg.explain_equivalence(s1, s2).show_expr(eg);
    };

    equate("(app (var s0) sym_x)", "sym_y", eg);
    eg.dump();
    explain("(app (var s0) sym_x)", "(app (var s1) sym_x)", eg);
}

#[test]
fn main8() {
    let mut eg: &mut EGraph<RiseENode> = &mut EGraph::new();
    let id = |s, eg: &mut EGraph<RiseENode>| -> AppliedId {
        let re = RecExpr::parse(s).unwrap();
        eg.add_syn_expr(re.clone())
    };

    let term = |s, eg: &mut EGraph<RiseENode>| -> RecExpr<RiseENode> {
        let re = RecExpr::parse(s).unwrap();
        eg.add_syn_expr(re.clone());
        re
    };

    let equate = |s1, s2, eg: &mut EGraph<RiseENode>| {
        let s1 = id(s1, eg);
        let s2 = id(s2, eg);
        eg.union(&s1, &s2);
    };

    let explain = |s1, s2, eg: &mut EGraph<RiseENode>| {
        let s1 = term(s1, eg);
        let s2 = term(s2, eg);
        eg.explain_equivalence(s1, s2).show_expr(eg);
    };

    equate("(app (app (var s0) (var s1)) sym_x)", "(app (app (var s1) (var s0)) sym_x)", eg);
    equate("(app (app (var s0) (var s1)) sym_y)", "(app (app (var s1) (var s0)) sym_y)", eg);
    equate("(app (app (var s0) (var s1)) sym_x)", "(app (app (var s0) (var s1)) sym_y)", eg);
    eg.dump();
    explain("(app (app (var s0) (var s1)) sym_x)", "(app (app (var s1) (var s0)) sym_y)", eg);
}

#[test]
fn main7() {
    let mut eg: &mut EGraph<RiseENode> = &mut EGraph::new();
    let id = |s, eg: &mut EGraph<RiseENode>| -> AppliedId {
        let re = RecExpr::parse(s).unwrap();
        eg.add_syn_expr(re.clone())
    };

    let term = |s, eg: &mut EGraph<RiseENode>| -> RecExpr<RiseENode> {
        let re = RecExpr::parse(s).unwrap();
        eg.add_syn_expr(re.clone());
        re
    };

    let equate = |s1, s2, eg: &mut EGraph<RiseENode>| {
        let s1 = id(s1, eg);
        let s2 = id(s2, eg);
        eg.union(&s1, &s2);
    };

    let explain = |s1, s2, eg: &mut EGraph<RiseENode>| {
        let s1 = term(s1, eg);
        let s2 = term(s2, eg);
        eg.explain_equivalence(s1, s2).show_expr(eg);
    };

    equate("(app (app (var s0) (var s1)) (var s2))", "(app (app (var s1) (var s0)) (var s2))", eg);
    equate("(app (app (var s0) (var s1)) (var s2))", "(app (app (var s0) (var s2)) (var s1))", eg);
    eg.dump();
    explain("(app (app (var s0) (var s1)) (var s2))", "(app (app (var s0) (var s1)) (var s2))", eg);
    explain("(app (app (var s0) (var s1)) (var s2))", "(app (app (var s0) (var s2)) (var s1))", eg);
    explain("(app (app (var s0) (var s1)) (var s2))", "(app (app (var s1) (var s0)) (var s2))", eg);
    explain("(app (app (var s0) (var s1)) (var s2))", "(app (app (var s1) (var s2)) (var s0))", eg);
    explain("(app (app (var s0) (var s1)) (var s2))", "(app (app (var s2) (var s0)) (var s1))", eg);
    explain("(app (app (var s0) (var s1)) (var s2))", "(app (app (var s2) (var s1)) (var s0))", eg);
}

#[test]
fn main6() {
    let mut eg: &mut EGraph<RiseENode> = &mut EGraph::new();
    let id = |s, eg: &mut EGraph<RiseENode>| -> AppliedId {
        let re = RecExpr::parse(s).unwrap();
        eg.add_syn_expr(re.clone())
    };

    let term = |s, eg: &mut EGraph<RiseENode>| -> RecExpr<RiseENode> {
        let re = RecExpr::parse(s).unwrap();
        eg.add_syn_expr(re.clone());
        re
    };

    let equate = |s1, s2, eg: &mut EGraph<RiseENode>| {
        let s1 = id(s1, eg);
        let s2 = id(s2, eg);
        eg.union(&s1, &s2);
    };

    let explain = |s1, s2, eg: &mut EGraph<RiseENode>| {
        let s1 = term(s1, eg);
        let s2 = term(s2, eg);
        eg.explain_equivalence(s1, s2).show_expr(eg);
    };

    equate("(app (var s0) (var s1))", "(app (var s1) (var s0))", eg);
    eg.dump();
    explain("(app (var s0) (var s1))", "(app (var s1) (var s0))", eg);
}

#[test]
fn main5() {
    let mut eg: &mut EGraph<RiseENode> = &mut EGraph::new();
    let id = |s, eg: &mut EGraph<RiseENode>| -> AppliedId {
        let re = RecExpr::parse(s).unwrap();
        eg.add_syn_expr(re.clone())
    };

    let term = |s, eg: &mut EGraph<RiseENode>| -> RecExpr<RiseENode> {
        let re = RecExpr::parse(s).unwrap();
        eg.add_syn_expr(re.clone());
        re
    };

    let equate = |s1, s2, eg: &mut EGraph<RiseENode>| {
        let s1 = id(s1, eg);
        let s2 = id(s2, eg);
        eg.union(&s1, &s2);
    };

    let explain = |s1, s2, eg: &mut EGraph<RiseENode>| {
        let s1 = term(s1, eg);
        let s2 = term(s2, eg);
        eg.explain_equivalence(s1, s2).show_expr(eg);
    };

    equate("(var s0)", "(app (var s0) sym_x)", eg);
    equate("sym_x", "sym_y", eg);
    eg.dump();
    explain("(var s2)", "(app (var s2) sym_y)", eg);
}

#[test]
fn main4() {
    let mut eg: &mut EGraph<TstENode> = &mut EGraph::new();
    let id = |s, eg: &mut EGraph<TstENode>| -> AppliedId {
        let re = RecExpr::parse(s).unwrap();
        eg.add_syn_expr(re.clone())
    };

    let term = |s, eg: &mut EGraph<TstENode>| -> RecExpr<TstENode> {
        let re = RecExpr::parse(s).unwrap();
        eg.add_syn_expr(re.clone());
        re
    };

    let equate = |s1, s2, eg: &mut EGraph<TstENode>| {
        let s1 = id(s1, eg);
        let s2 = id(s2, eg);
        eg.union(&s1, &s2);
    };

    let explain = |s1, s2, eg: &mut EGraph<TstENode>| {
        let s1 = term(s1, eg);
        let s2 = term(s2, eg);
        eg.explain_equivalence(s1, s2).show();
    };

    equate("(f s1 s2)", "(g s2 s1)", eg);
    equate("(g s1 s2)", "(h s1 s2)", eg);
    eg.dump();
    explain("(f s1 s2)", "(h s2 s1)", eg);
}

#[test]
fn main3() {
    let mut eg: &mut EGraph<RiseENode> = &mut EGraph::new();
    let id = |s, eg: &mut EGraph<RiseENode>| {
        let re = RecExpr::parse(s).unwrap();
        eg.add_syn_expr(re.clone())
    };

    let term = |s, eg: &mut EGraph<RiseENode>| {
        let re = RecExpr::parse(s).unwrap();
        eg.add_syn_expr(re.clone());
        re
    };

    let x1 = id("sym_x1", eg);
    let x2 = id("sym_x2", eg);
    let x1x3 = term("(app sym_x1 sym_x3)", eg);
    let x2x3 = term("(app sym_x2 sym_x3)", eg);
    eg.union(&x1, &x2);
    eg.dump();
    dbg!(&x1x3);
    dbg!(&x2x3);
    eg.explain_equivalence(x1x3, x2x3).show();
}

#[test]
fn main2() {
    let p = |s| RecExpr::parse(s).unwrap();
    let x1 = p("sym_x1");
    let x2 = p("sym_x2");
    let x3 = p("sym_x3");
    let x4 = p("sym_x4");
    let mut eg: EGraph<RiseENode> = EGraph::new();
    let y1 = eg.add_expr(x1.clone());
    let y2 = eg.add_expr(x2.clone());
    let y3 = eg.add_expr(x3.clone());
    let y4 = eg.add_expr(x4.clone());
    eg.union(&y1, &y2);
    eg.union(&y3, &y4);
    eg.union(&y2, &y3);
    eg.explain_equivalence(x1, x4).show();
}
