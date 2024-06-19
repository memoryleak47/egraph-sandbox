use crate::*;

fn assert_reaches(start: &str, goal: &str, steps: usize) {
    let start = array_parse(start);
    let goal = array_parse(goal);

    let rules = array_rules();

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
    assert!(false);
}

#[test]
fn tile_1d() {
    let start = "(m (* n1 32) f)";
    let goal = "(o j (o (m n1 (m 32 f)) (s 32)))";
    assert_reaches(start, goal, 40);
}

#[test]
fn tile_3d() {
    let start = "(m (* n1 32) (m (* n2 32) (m (* n3 32) f)))";
    let goal = "(m (* n1 31) f)";

    let split_goals: &[&str] = &[
        "(o (o j (m n1 (m 32 (m (* n2 32) (m (* n3 32) f))))) (s 32))",
        "(o (m (* n1 32) j) (o (m (* n1 32) (m n2 (m 32 (m (* n3 32) f)))) (m (* n1 32) (s 32))))",
        "(o (m (* n1 32) (m (* n2 32) j)) (o (m (* n1 32) (m (* n2 32) (m n3 (m 32 f)))) (m (* n1 32) (m (* n2 32) (s 32)))))",
        "(o (o j (m n1 (m 32 (o j (o (m n2 (m 32 (m (* n3 32) f))) (s 32)))))) (s 32))",
        "(m (* n1 32) (o j (o (m n2 (m 32 (o j (o (m n3 (m 32 f)) (s 32))))) (s 32))))",
        "(o (o j (m n1 (m 32 (o j (o (m n2 (m 32 (o j (o (m n3 (m 32 f)) (s 32))))) (s 32)))))) (s 32))",
    ];


    let goals: &[&str] = &[
        "(o j (o (m n1 (m 32 (m (* n2 32) (m (* n3 32) f)))) (s 32)))",
        "(o (m (* n1 32) j) (o (m (* n1 32) (m n2 (m 32 (m (* n3 32) f)))) (m (* n1 32) (s 32))))",
        "(o (o (m (* n1 32) (m (* n2 32) j)) (m (* n1 32) (m (* n2 32) (m n3 (m 32 f))))) (m (* n1 32) (m (* n2 32) (s 32))))",
        "(o j (o (m n1 (o (m 32 j) (o (o T (m n2 (m 32 (m 32 (m (* n3 32) f))))) (o T (m 32 (s 32)))))) (s 32)))",
        "(m (* n1 32) (o j (o (m n2 (o (o (o (m 32 j) T) (m n3 (m 32 (m 32 f)))) (o T (m 32 (s 32))))) (s 32))))",
        "(o j (o (m n1 (o (o (o (m 32 j) T) (o (m n2 (o (m 32 (o (m 32 j) T)) (o (o T (o (m n3 (m 32 (m 32 (m 32 f)))) T)) (m 32 (o T (m 32 (s 32))))))) T)) (m 32 (s 32)))) (s 32)))",
    ];

    assert_reaches(start, split_goals[0], 40);
}
