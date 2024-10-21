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

use memory_stats::memory_stats;
use std::time::Instant;

fn assert_reaches<W>(start: &str, goal: &str, mut csv_out: W, steps: usize) where W: std::io::Write {
    let start = RecExpr::parse(start).unwrap();
    let goal = RecExpr::parse(goal).unwrap();

    let rules = rise_rules(RiseSubstMethod::SmallStep);

    let mut eg = EGraph::new();
    let i1 = eg.add_expr(start.clone());
    for iteration in 0..steps {
        let start_time = Instant::now();
        apply_rewrites(&mut eg, &rules);
        if let Some(i2) = lookup_rec_expr(&goal, &eg) {
            if eg.eq(&i1, &i2) {
                dbg!(eg.total_number_of_nodes());
                #[cfg(feature = "explanations")]
                println!("{}", eg.explain_equivalence(start, goal).to_string(&eg));
                iteration_stats(&mut csv_out, iteration, &eg, true, start_time);
                return;
            }
        }
        let out_of_memory = iteration_stats(&mut csv_out, iteration, &eg, false, start_time);
        if out_of_memory {
            dbg!("reached memory limit!");
            break;
        }
    }

    dbg!(extract::<_, _, AstSizeNoLet>(i1, &eg));
    dbg!(&goal);
    assert!(false);
}


// iteration number,
// physical memory,
// virtual memory,
// e-graph nodes (hashcons size),
// e-graph nodes (computed),
// e-graph classes,
// total time,
// found
fn iteration_stats<W, L, N>(csv_out: &mut W, it_number: usize, eg: &EGraph<L, N>, found: bool, start_time: Instant) -> bool
    where W: std::io::Write, L: Language, N: Analysis<L>
{
    let memory = memory_stats().expect("could not get current memory usage");
    let out_of_memory = memory.virtual_mem > 4_000_000_000;
    writeln!(csv_out, "{}, {}, {}, {}, {}, {}, {}, {}",
        it_number,
        memory.physical_mem,
        memory.virtual_mem,
        eg.total_number_of_nodes(),
        eg.ids().into_iter().map(|c| eg.enodes(c).len()).sum::<usize>(),
        eg.ids().len(),
        start_time.elapsed().as_secs_f64(),
        found);
    out_of_memory
}

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let lhs = &args[0];
    let rhs = &args[1];
    let csv_out = &args[2];
    let mut csv_f = std::fs::File::create(csv_out).unwrap();
    assert_reaches(lhs, rhs, csv_f, 60);
}
