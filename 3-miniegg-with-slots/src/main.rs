use miniegg_with_slots::*;

fn assert_reaches(start: RecExpr<RiseENode>, goal: RecExpr<RiseENode>, steps: usize) {
    let rules = rise_rules(SubstMethod::SmallStep);

    let mut eg = EGraph::new().with_explanations_enabled();
    let i1 = eg.add_expr(start.clone());
    for _ in 0..steps {
        do_rewrites(&mut eg, &rules);
        dbg!(eg.total_number_of_nodes());
        if let Some(i2) = lookup_rec_expr(&goal, &eg) {
            if eg.eq(&i1, &i2) {
                dbg!(eg.total_number_of_nodes());
                dbg!(eg.explain_equivalence(start, goal));
                return;
            }
        }
    }

    dbg!(extract::<_, AstSizeNoLet>(i1, &eg));
    dbg!(&goal);
    assert!(false);
}

fn main() {
    let p = "(app (lam s0 (app (var s0) (var s0))) (lam s1 (var s1)))";
    let q = "(lam s0 (var s0))";

    let p = RecExpr::parse(p).unwrap();
    let q = RecExpr::parse(q).unwrap();
    assert_reaches(p, q, 40);
}
