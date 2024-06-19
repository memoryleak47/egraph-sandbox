use crate::*;

fn assert_reaches(start: &str, goal: &str, steps: usize, extra_rules: &[&'static str]) {
    let start = array_parse(start);
    let goal = array_parse(goal);

    let rules = array_rules(extra_rules);

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
    assert_reaches(start, goal, 40, &["transpose-maps", "split-map"]);
}

#[test]
fn tile_2d() {
    let start = "(m (* n1 32) (m (* n2 32) f))";
    let mid = "(o (o (o (m (* n1 32) j) j) (o (m n1 (m 32 (m n2 (m 32 f)))) (m n1 (m 32 (s 32))))) (s 32))";
    let goal = "(o (o (o (o (m (* n1 32) j) j) (m n1 T)) (o (m n1 (m n2 (m 32 (m 32 f)))) (m n1 T))) (o (m n1 (m 32 (s 32))) (s 32)))";

    assert_reaches(start, mid, 40, &["transpose-maps", "split-map"]);
}

#[test]
fn tile_3d() {
    let start = "(m (* n1 32) (m (* n2 32) (m (* n3 32) f)))";
    let mid = "(o (m (* n1 32) (o (m (* n2 32) j) j)) (o (o j (o (m n1 (m 32 (m n2 (m 32 (m n3 (m 32 f)))))) (s 32))) (m (* n1 32) (o (m n2 (m 32 (s 32))) (s 32)))))";
    let goal = "(o (o (m (* n1 32) (o (m (* n2 32) j) j)) j) (o (o (m n1 (o T (m n2 (o (m 32 T) T)))) (o (m n1 (m n2 (m n3 (m 32 (m 32 (m 32 f)))))) (m n1 (m n2 T)))) (o (o (m n1 (o (m n2 (m 32 T)) T)) (s 32)) (m (* n1 32) (o (m n2 (m 32 (s 32))) (s 32))))))";

    assert_reaches(start, mid, 40, &["transpose-maps", "split-map"]);
}
