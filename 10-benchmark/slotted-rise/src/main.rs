mod tst;
pub use tst::*;

mod rewrite;
pub use rewrite::*;

mod my_cost;
pub use my_cost::*;

mod my_cost2;
pub use my_cost2::*;

mod lang;
pub use lang::*;

pub use symbol_table::GlobalSymbol as Symbol;
pub use slotted_egraphs::*;
pub use std::ops::RangeInclusive;

// (a ° b)
fn comp(a: String, b: String) -> String {
    let x = Slot::fresh();
    format!("(lam {x} (app {a} (app {b} (var {x}))))")
}

// (map x)
fn map(x: String) -> String {
    format!("(app map {x})")
}

// f1 ° ... ° fm
fn chained_fns(it: impl Iterator<Item=usize>) -> String {
    let mut it = it.map(|x| format!("f{x}"));

    let mut out = it.next().unwrap();
    for i in it {
        out = comp(i, out);
    }
    out
}

fn nested_maps(n: usize, arg: String) -> String {
    let mut out = arg;
    for _ in 0..n {
        out = map(out);
    }
    out
}

// N = number of nested maps.
// M = half amount of the chained functions.
fn generate_lhs(n: usize, m: usize) -> String {
    let l = chained_fns(1..=m);
    let r = chained_fns(m..=(2*m));
    let out = comp(l, r);
    let out = nested_maps(n, out);
    out
}

fn generate_rhs(n: usize, m: usize) -> String {
    let l = nested_maps(n, chained_fns(1..=m));
    let r = nested_maps(n, chained_fns(m..=(2*m)));
    let out = comp(l, r);
    out
}

fn assert_reaches(start: &str, goal: &str, steps: usize) {
    let start = RecExpr::parse(start).unwrap();
    let goal = RecExpr::parse(goal).unwrap();

    let rules = rise_rules(SubstMethod::SmallStep);

    let mut eg = EGraph::new();
    let i1 = eg.add_expr(start.clone());
    for _ in 0..steps {
        apply_rewrites(&mut eg, &rules);
        dbg!(eg.total_number_of_nodes());
        if let Some(i2) = lookup_rec_expr(&goal, &eg) {
            if eg.eq(&i1, &i2) {
                dbg!(eg.total_number_of_nodes());
                #[cfg(feature = "explanations")]
                println!("{}", eg.explain_equivalence(start, goal).to_string(&eg));
                return;
            }
        }
    }

    dbg!(extract::<_, AstSizeNoLet>(i1, &eg));
    dbg!(&goal);
    assert!(false);
}

fn run(n: usize, m: usize) {
    let lhs = generate_lhs(n, m);
    let rhs = generate_rhs(n, m);
    dbg!(&lhs);
    dbg!(&rhs);
    assert_reaches(&lhs, &rhs, 60);
}

fn main() {
    run(3, 3);
}
