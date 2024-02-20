mod term;
use term::*;

mod cost;
use cost::*;

#[allow(unused)] mod subst1;
#[allow(unused)] use subst1::*;

#[allow(unused)] mod subst2;
#[allow(unused)] use subst2::*;

#[allow(unused)] mod subst3;
#[allow(unused)] use subst3::*;

use egg::*;

fn make_rules() -> Vec<Rewrite<Term, ()>> {
    vec![
        // subst1(),
        // subst2(),
        subst3(),
        rewrite!("mul-0"; "(* ?a 0)" => "0"),
        rewrite!("mul-1"; "(* ?a 1)" => "?a"),
        rewrite!("mul-comm"; "(* ?a ?b)" => "(* ?b ?a)"),
        rewrite!("mul-assoc"; "(* ?a (* ?b ?c))" => "(* (* ?a ?b) ?c)"),

        rewrite!("add-0"; "(+ ?a 0)" => "?a"),
        rewrite!("add-comm"; "(+ ?a ?b)" => "(+ ?b ?a)"),
        rewrite!("add-assoc"; "(+ ?a (+ ?b ?c))" => "(+ (+ ?a ?b) ?c)"),

        rewrite!("distr"; "(* (+ ?a ?b) ?c)" => "(+ (* ?a ?c) (* ?b ?c))"),
    ]
}

fn main() {
    assert_eq!(simplify("(app (lam v b) c)"), "b");
    assert_eq!(simplify("(app (lam v v) c)"), "c");

    let omega = "(lam x (app x x))";
    let infinite_loop = format!("(app {omega} {omega})");

    let id = "(lam x x)";
    let t = "(lam x (lam y x))";
    let s = format!("(app (app {t} {id}) {infinite_loop})");

    assert_eq!(simplify(&infinite_loop), infinite_loop);

    assert_eq!(simplify("(+ x (* 2 0))"), "x");

    assert_eq!(simplify("(app (app (lam x (lam y x)) a1) a2)"), "a1");
    assert_eq!(simplify("(app (app (lam x (lam y y)) a1) a2)"), "a2");

    assert_eq!(simplify(&s), "(lam x x)");

    let p = "(app (lam x (lam y x)) y)";
    let p = format!("(app {p} a)");
    assert_eq!(simplify(&p), "y");

    // Y-combinator example.
    // translating church numerals to numbers.
    let a = "(lam x (app f (app x x)))";
    let y = format!("(lam f (app {a} {a}))");

    let zero = "(lam z (lam s z))";
    let suc = "(lam arg (lam z (lam s (app s arg))))";

    let d = "(lam a (+ (app f a) 1))";
    let translate_impl = format!("(lam f (lam n (app (app n 0) {d})))");
    let translate = format!("(lam n (app {y} {translate_impl}))");

    // the number 2.
    let rhs = format!("(app {suc} (app {suc} {zero}))");
    let s = format!("(app {translate} {rhs})");
    assert_eq!(simplify(&s), "(+ 1 1)");
}

fn simplify(s: &str) -> String {
    let expr: RecExpr<Term> = s.parse().unwrap();
    let runner = Runner::default().with_expr(&expr).run(&make_rules());
    let root = runner.roots[0];
    let extractor = Extractor::new(&runner.egraph, MyAstSize);
    let (best_cost, best) = extractor.find_best(root);
    println!("Simplified {} to {} with cost {:?}", expr, best, best_cost);
    best.to_string()
}
