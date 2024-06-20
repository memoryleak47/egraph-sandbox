use miniegg_with_slots::*;

fn main() {
    run("binomial", WithExpansion::No);
}

fn run(name: &str, exp: WithExpansion) {
    let mut rules = vec!["beta", "eta"];

    if let WithExpansion::Yes = exp {
        rules.push("eta-expansion");
    }

    match name {
        "reduction" => {
            let start = "(app (lam compose (app (lam add1 (app (app (var compose) (var add1)) (app (app (var compose) (var add1)) (app (app (var compose) (var add1)) (app (app (var compose) (var add1)) (app (app (var compose) (var add1)) (app (app (var compose) (var add1)) (var add1)))))))) (lam y (app (app add (var y)) 1)))) (lam f (lam g (lam x (app (var f) (app (var g) (var x)))))))".into();
            let goal = "(lam x (app (app add (app (app add (app (app add (app (app add (app (app add (app (app add (app (app add (var x)) 1)) 1)) 1)) 1)) 1)) 1)) 1))".into();
            bench(start, goal, &rules, false)
        },
        "fission" => {
            let start = "(lam f1 (lam f2 (lam f3 (lam f4 (lam f5 (app map (lam x3 (app (var f5) (app (lam x2 (app (var f4) (app (lam x1 (app (var f3) (app (lam x0 (app (var f2) (app (var f1) (var x0)))) (var x1)))) (var x2)))) (var x3))))))))))".into();
            let goal = "(lam f1 (lam f2 (lam f3 (lam f4 (lam f5 (lam x7 (app (app map (lam x6 (app (var f5) (app (lam x5 (app (var f4) (app (var f3) (var x5)))) (var x6))))) (app (app map (lam x4 (app (var f2) (app (var f1) (var x4))))) (var x7)))))))))".into();
            rules.extend(["map-fusion", "map-fission"]);
            bench(start, goal, &rules, true)
        },
        "binomial" => {
            // DOESN'T WORK:
            // let start =  "(lam x5 (app (app map (app map (lam nbh (app (app (lam a (lam b (app (app (app reduce add) 0) (app (app map (lam mt (app (app mul (app fst (var mt))) (app snd (var mt))))) (app (app zip (var a)) (var b)))))) (app join weights2d)) (app join (var nbh)))))) (app (lam x4 (app (app map transpose) (app (lam x3 (app (app (app slide 3) 1) (app (app map (app (app slide 3) 1)) (var x3)))) (var x4)))) (var x5))))".into();
            // let goal = "(lam x9 (app (app map (lam x8 (app (app map (app (lam x0 (lam x1 (app (app (app reduce add) 0) (app (app map (lam x2 (app (app mul (app fst (var x2))) (app snd (var x2))))) (app (app zip (var x0)) (var x1)))))) weightsH)) (app (lam x7 (app (app (app slide 3) 1) (app (lam x6 (app (app map (app (lam a (lam b (app (app (app reduce add) 0) (app (app map (lam mt (app (app mul (app fst (var mt))) (app snd (var mt))))) (app (app zip (var a)) (var b)))))) weightsV)) (app transpose (var x6)))) (var x7)))) (var x8))))) (app (app (app slide 3) 1) (var x9))))".into();

            // WORKS:
            let start = "(app (app map (app map (lam s0 (app (app (app reduce add) 0) (app (app map (lam s-1 (app (app mul (app fst (var s-1))) (app snd (var s-1))))) (app (app zip (app join weights2d)) (app join (var s0)))))))) (app (app map transpose) (app (app (app slide 3) 1) (app (app map (app (app slide 3) 1)) input))))".into();
            let goal = "(app (app map (lam s0 (app (app map (lam s1 (app (app (app reduce add) 0) (app (app map (lam s-2 (app (app mul (app fst (var s-2))) (app snd (var s-2))))) (app (app zip weightsH) (var s1)))))) (app (app (app slide 3) 1) (app (app map (lam s2 (app (app (app reduce add) 0) (app (app map (lam s-3 (app (app mul (app fst (var s-3))) (app snd (var s-3))))) (app (app zip weightsV) (var s2)))))) (app transpose (var s0))))))) (app (app (app slide 3) 1) input))".into();

            rules.extend([
                "remove-transpose-pair", "map-fusion", "map-fission",
                "slide-before-map", "map-slide-before-transpose", "slide-before-map-map-f",
                "separate-dot-vh-simplified", "separate-dot-hv-simplified"
            ]);
            bench(start, goal, &rules, true)
        },
        _ => panic!("did not expect {}", name)
    }
}

fn bench(start: &str, goal: &str, rules: &[&str], normalize: bool) {
    let start = RecExpr::parse(&slottify(start.into()).0).unwrap();
    let goal = RecExpr::parse(&slottify(goal.into()).0).unwrap();

    assert_reaches(start, goal, rules, 40);
}

fn assert_reaches(start: RecExpr<RiseENode>, goal: RecExpr<RiseENode>, rules: &[&str], steps: usize) {
    let rules = rise_rules(rules);

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
