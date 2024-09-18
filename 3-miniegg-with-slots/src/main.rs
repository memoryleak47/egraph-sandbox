use miniegg_with_slots::*;

fn main() {
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
    dbg!(eg.explain_equivalence(x1, x4));
}
