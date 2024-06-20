use miniegg_with_slots::*;

fn main() {
    run(binomial_problem(), WithExpansion::Yes);
}

fn run((start, goal): Problem, exp: WithExpansion) {
    assert_reaches(start, goal, 40, exp);
}

fn assert_reaches(start: RecExpr<RiseENode>, goal: RecExpr<RiseENode>, steps: usize, exp: WithExpansion) {
    let rules = rise_rules(exp);

    let mut eg = EGraph::new();
    let i1 = eg.add_expr(start);
    for _ in 0..steps {
        do_rewrites(&mut eg, &rules);
        dbg!(eg.total_number_of_nodes());
        if let Some(i2) = lookup_rec_expr(&goal, &eg) {
            if eg.eq(&i1, &i2) {
                dbg!(eg.total_number_of_nodes());
                return;
            }
        }
    }

    dbg!(extract::<_, AstSizeNoLet>(i1, &eg));
    dbg!(&goal);
    panic!("You've hit the iteration limit!");
}
