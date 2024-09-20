use miniegg_with_slots::*;

fn main() {
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
