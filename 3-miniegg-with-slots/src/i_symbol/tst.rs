use crate::*;

fn symbol_rules(extra_rules: &[&'static str]) -> Vec<Rewrite<SymbolENode>> {
    let mut rewrites = Vec::new();

    // assoc:
    rewrites.push(rew("(o (o ?f ?g) ?h)", "(o ?f (o ?g ?h))"));
    rewrites.push(rew("(o ?f (o ?g ?h))", "(o (o ?f ?g) ?h)"));

    // map-fusion:
    // rewrites.push(rew("(o (m ?n ?f) (m ?n ?g))", "(m ?n (o ?f ?g))"));

    // map-fission:
    rewrites.push(rew("(m ?n (o ?f ?g))", "(o (m ?n ?f) (m ?n ?g))"));

    for r in extra_rules {
        let rewrite = match *r {
            "transpose-maps" => rew("(m ?n1 (m ?n2 ?f))", "(o T (o (m ?n2 (m ?n1 ?f)) T))"),
            "split-map" => rew("(m (* ?n1 ?n2) ?f)", "(o j (o (m ?n1 (m ?n2 ?f)) (s ?n2)))"),
            x => panic!("unknown rule: {x}"),
        };
        rewrites.push(rewrite);
    }

    rewrites
}

fn rew(s1: &str, s2: &str) -> Rewrite<SymbolENode> {
    let pat = Pattern::parse(s1).unwrap();
    let outpat = Pattern::parse(s2).unwrap();

    mk_rewrite(pat, outpat)
}

fn assert_reaches(start: &str, goal: &str, steps: usize, extra_rules: &[&'static str]) {
    let start = RecExpr::parse(start).unwrap();
    let goal = RecExpr::parse(goal).unwrap();

    let rules = symbol_rules(extra_rules);

    let mut eg = EGraph::new().with_explanations_enabled();
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

    dbg!(extract::<_, AstSize>(i1, &eg));
    dbg!(&goal);
    assert!(false);
}

#[test]
fn sym_tile_1d() {
    let start = "(m (* n1 32) f)";
    let goal = "(o j (o (m n1 (m 32 f)) (s 32)))";
    assert_reaches(start, goal, 40, &["transpose-maps", "split-map"]);
}

#[test]
fn sym_tile_2d() {
    let start = "(m (* n1 32) (m (* n2 32) f))";
    let mid = "(o (o (o (m (* n1 32) j) j) (o (m n1 (m 32 (m n2 (m 32 f)))) (m n1 (m 32 (s 32))))) (s 32))";
    let goal = "(o (o (o (o (m (* n1 32) j) j) (m n1 T)) (o (m n1 (m n2 (m 32 (m 32 f)))) (m n1 T))) (o (m n1 (m 32 (s 32))) (s 32)))";

    assert_reaches(start, mid, 40, &["split-map"]);
    assert_reaches(mid, goal, 40, &["transpose-maps"]);
}

#[test]
fn sym_tile_3d() {
    let start = "(m (* n1 32) (m (* n2 32) (m (* n3 32) f)))";
    let mid = "(o (m (* n1 32) (o (m (* n2 32) j) j)) (o (o j (o (m n1 (m 32 (m n2 (m 32 (m n3 (m 32 f)))))) (s 32))) (m (* n1 32) (o (m n2 (m 32 (s 32))) (s 32)))))";
    let goal = "(o (o (m (* n1 32) (o (m (* n2 32) j) j)) j) (o (o (m n1 (o T (m n2 (o (m 32 T) T)))) (o (m n1 (m n2 (m n3 (m 32 (m 32 (m 32 f)))))) (m n1 (m n2 T)))) (o (o (m n1 (o (m n2 (m 32 T)) T)) (s 32)) (m (* n1 32) (o (m n2 (m 32 (s 32))) (s 32))))))";

    assert_reaches(start, mid, 40, &["split-map"]);
    assert_reaches(mid, goal, 40, &["transpose-maps"]);
}
