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

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let lhs = &args[0];
    let rhs = &args[1];
    assert_reaches(lhs, rhs, 60);
}
