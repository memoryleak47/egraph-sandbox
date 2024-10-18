mod rise;
mod rules;
mod alpha_equiv;
mod dbrise;
mod dbrules;

enum WithExpansion { Yes, No }

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let name = &*args[0];
    let binding = &*args[1];

    let mut rules = vec!["beta", "eta"];

    if let Some("eta-exp") = args.get(2).map(|x| &**x) {
        rules.push("eta-expansion");
    }

    let bench = |start, goal, rules| {
        bench_prove_equiv(name, start, goal, rules, binding);
    };

    match name {
        "reduction" => {
            let start = "(app (lam compose (app (lam add1 (app (app (var compose) (var add1)) (app (app (var compose) (var add1)) (app (app (var compose) (var add1)) (app (app (var compose) (var add1)) (app (app (var compose) (var add1)) (app (app (var compose) (var add1)) (var add1)))))))) (lam y (app (app add (var y)) 1)))) (lam f (lam g (lam x (app (var f) (app (var g) (var x)))))))";
            let goal = "(lam x (app (app add (app (app add (app (app add (app (app add (app (app add (app (app add (app (app add (var x)) 1)) 1)) 1)) 1)) 1)) 1)) 1))";
            bench(start, goal, &rules)
        },
        "fission" => {
            let start = "(lam f1 (lam f2 (lam f3 (lam f4 (lam f5 (app map (lam x11 (app (var f5) (app (var f4) (app (var f3) (app (var f2) (app (var f1) (var x11)))))))))))))";
            let goal =  "(lam f1 (lam f2 (lam f3 (lam f4 (lam f5 (lam x7 (app (app map (lam x6 (app (var f5) (app (var f4) (app (var f3) (var x6)))))) (app (app map (lam x4 (app (var f2) (app (var f1) (var x4))))) (var x7)))))))))";
            rules.extend(["map-fusion", "map-fission"]);
            bench(start, goal, &rules)
        },
        "binomial" => {
            let start = "(lam x17 (app (app map (app map (lam nbh (app (app (app reduce add) 0) (app (app map (lam mt (app (app mul (app fst (var mt))) (app snd (var mt))))) (app (app zip (app join weights2d)) (app join (var nbh)))))))) (app (app map transpose) (app (app (app slide 3) 1) (app (app map (app (app slide 3) 1)) (var x17))))))";
            let goal = "(lam x26 (app (app map (lam x25 (app (app map (lam x15 (app (app (app reduce add) 0) (app (app map (lam x16 (app (app mul (app fst (var x16))) (app snd (var x16))))) (app (app zip weightsH) (var x15)))))) (app (app (app slide 3) 1) (app (app map (lam b (app (app (app reduce add) 0) (app (app map (lam mt (app (app mul (app fst (var mt))) (app snd (var mt))))) (app (app zip weightsV) (var b)))))) (app transpose (var x25))))))) (app (app (app slide 3) 1) (var x26))))";

            rules.extend([
                "remove-transpose-pair", "map-fusion", "map-fission",
                "slide-before-map", "map-slide-before-transpose", "slide-before-map-map-f",
                "separate-dot-vh-simplified", "separate-dot-hv-simplified"
            ]);
            bench(start, goal, &rules)
        },
        _ => panic!("did not expect {}", name)
    }
}

use std::env;
use egg::*;
use crate::rise::*;
use crate::dbrise::*;
use crate::rules::*;
use crate::dbrules::*;
// use crate::scheduler::*;
use crate::alpha_equiv::*;
use crate::dbrise::DBRiseExpr;

fn to_db_str<S: AsRef<str>>(e: S) -> String {
    format!("{}", to_db(e.as_ref().parse().unwrap()))
}

fn to_db(e: RecExpr<Rise>) -> DBRiseExpr {
    let e = e.as_ref();
    let mut r = vec![];
    rec(&mut r, e, e.len() - 1, &[]);

    fn rec(r: &mut Vec<DBRise>, expr: &[Rise], i: usize, bound: &[Symbol]) -> Id {
        match expr[i] {
            Rise::Number(n) => add(r, DBRise::Number(n)),
            Rise::Symbol(s) => add(r, DBRise::Symbol(s)),
            Rise::Var(x) => {
                let xs = unwrap_symbol(&expr[usize::from(x)]);
                let pos = bound.iter().position(|&s| s == xs)
                    .unwrap_or_else(|| panic!("{} not bound", xs));
                add(r, DBRise::Var(Index(pos as u32)))
            },
            Rise::Lambda([v, e]) => {
                let mut bound2 = vec![unwrap_symbol(&expr[usize::from(v)])];
                bound2.extend_from_slice(bound);
                let e2 = rec(r, expr, usize::from(e), &bound2[..]);
                add(r, DBRise::Lambda(e2))
            }
            Rise::App([f, e]) => {
                let f2 = rec(r, expr, usize::from(f), bound);
                let e2 = rec(r, expr, usize::from(e), bound);
                add(r, DBRise::App([f2, e2]))
            }
            Rise::Let([_, _, _]) => unimplemented!(),
            Rise::Then(_) => unimplemented!()
        }
    }

    r.into()
}

fn bench_prove_equiv(name: &str, start_s: &str, goal_s: &str, rule_names: &[&str], binding: &str) {
    println!();
    println!("-------");
    println!("- goal:         {}", name);
    println!("- binding:      {}", binding);
    println!("-------");
    println!();

    let start_p: RecExpr<Rise> = start_s.parse().unwrap();
    let goal_p: RecExpr<Rise> = goal_s.parse().unwrap();
    let start = start_p;
    let goal = goal_p;
    println!("start: {}", start);
    println!("goal: {}", goal);

    let mut rule_names: Vec<&str> = rule_names.iter().cloned().collect();
    match binding {
        "name" => {
            if rule_names.contains(&"beta") {
                rule_names.extend([
                    "opt:let-unused",
                    "opt:let-app", "opt:let-var-same",
                    "opt:let-lam-same", "opt:let-lam-diff",
                ]);
            }

            prove_equiv_aux(start, goal, rules(&*rule_names))
        },
        "de-bruijn" => {
            if rule_names.contains(&"beta") {
                rule_names.extend([
                    "sig-unused", "phi-unused",
                    "sig-lam", "sig-app", "sig-var-const",
                    "phi-lam", "phi-app", "phi-var-const"
                ]);
            }

            to_db_prove_equiv_aux(start, goal, dbrules(&*rule_names))
        },
        _ => panic!("did not expect {}", binding)
    }

    println!();
}

fn prove_equiv_aux(start: RecExpr<Rise>, goal: RecExpr<Rise>, rules: Vec<Rewrite<Rise, RiseAnalysis>>) {
    let goal = expr_to_alpha_equiv_pattern(goal);
    let goals: Vec<Pattern<Rise>> = vec![goal];
    let mut runner = Runner::default()
        .with_expr(&start);

    // NOTE this is a bit of hack, we rely on the fact that the
    // initial root is the last expr added by the runner. We can't
    // use egraph.find_expr(start) because it may have been pruned
    // away
    let id = runner.egraph.find(*runner.roots.last().unwrap());

    let goals2 = goals.clone();
    runner = runner
        .with_scheduler(SimpleScheduler)
        .with_node_limit(100_000_000)
        .with_iter_limit(500)
        .with_time_limit(std::time::Duration::from_secs(60*60)) // 4mn
        .with_hook(move |r| {
            dbg!(r.egraph.total_number_of_nodes());
            if goals2.iter().all(|g| g.search_eclass(&r.egraph, id).is_some()) {
                Err("Done".into())
            } else {
                Ok(())
            }
        }).run(&rules);
    runner.print_report();
    let rules = runner.iterations.iter().map(|i|
        i.applied.iter().map(|(_, n)| n).sum::<usize>()).sum::<usize>();
    println!("applied rules: {}", rules);
    runner.iterations.iter().for_each(|i| println!("{:?}", i));
    // count_alpha_equiv(&mut runner.egraph);
    // runner.egraph.dot().to_svg(format!("/tmp/{}.svg", name)).unwrap();
    runner.egraph.check_goals(id, &goals);
}

fn db_prove_equiv_aux(start: RecExpr<DBRise>, goal: RecExpr<DBRise>, rules: Vec<Rewrite<DBRise, DBRiseAnalysis>>) {
    let goals: Vec<Pattern<DBRise>> = vec![goal.as_ref().into()];
    let mut runner = Runner::default()
        .with_expr(&start);

    // NOTE this is a bit of hack, we rely on the fact that the
    // initial root is the last expr added by the runner. We can't
    // use egraph.find_expr(start) because it may have been pruned
    // away
    let id = runner.egraph.find(*runner.roots.last().unwrap());

    let goals2 = goals.clone();
    runner = runner
        .with_scheduler(SimpleScheduler)
        .with_node_limit(100_000_000)
        .with_iter_limit(500)
        .with_time_limit(std::time::Duration::from_secs(60*60)) // 4mn
        .with_hook(move |r| {
            dbg!(r.egraph.total_number_of_nodes());
            if goals2.iter().all(|g| g.search_eclass(&r.egraph, id).is_some()) {
                Err("Done".into())
            } else {
                Ok(())
            }
        }).run(&rules);
    runner.print_report();
    let rules = runner.iterations.iter().map(|i|
        i.applied.iter().map(|(_, n)| n).sum::<usize>()).sum::<usize>();
    println!("applied rules: {}", rules);
    runner.iterations.iter().for_each(|i| println!("{:?}", i));
    runner.egraph.check_goals(id, &goals);
}

fn to_db_prove_equiv_aux(start: RecExpr<Rise>, goal: RecExpr<Rise>, rules: Vec<Rewrite<DBRise, DBRiseAnalysis>>) {
    let start_db = to_db(start);
    let goal_db = to_db(goal);
    println!("start (db): {}", start_db);
    println!("goal  (db): {}", goal_db);
    db_prove_equiv_aux(start_db, goal_db, rules)
}
