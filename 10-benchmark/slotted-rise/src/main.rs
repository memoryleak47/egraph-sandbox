mod rewrite;
pub use rewrite::*;

mod my_cost;
pub use my_cost::*;

mod my_cost2;
pub use my_cost2::*;

mod lang;
pub use lang::*;

mod dblang;
pub use dblang::*;

#[cfg(feature = "trace")]
mod trace;
#[cfg(feature = "trace")]
use trace::BucketSubscriber;

pub use symbol_table::GlobalSymbol as Symbol;
pub use slotted_egraphs::{*, Id};
pub use std::ops::RangeInclusive;

use memory_stats::memory_stats;
use std::time::Instant;

use tracing::*;

fn assert_reaches<W>(start: &str, goal: &str, mut csv_out: W, steps: usize) where W: std::io::Write {
    let init_span = trace_span!("init").entered();
    let start = RecExpr::parse(start).unwrap();
    let goal = RecExpr::parse(goal).unwrap();

    let rules = rise_rules(RiseSubstMethod::SmallStep);

    let mut eg = EGraph::new();
    let i1 = eg.add_expr(start.clone());
    init_span.exit();
    for iteration in 0..steps {
        dbg!(eg.total_number_of_nodes());
        let start_time = Instant::now();
        apply_rewrites(&mut eg, &rules);
        let check_span = trace_span!("check").entered();
        if let Some(i2) = lookup_rec_expr(&goal, &eg) {
            if eg.eq(&i1, &i2) {
                dbg!(eg.total_number_of_nodes());
                #[cfg(feature = "explanations")]
                println!("{}", eg.explain_equivalence(start, goal).to_string(&eg));
                iteration_stats(&mut csv_out, iteration, &eg, true, start_time);
                return;
            }
        }
        check_span.exit();
        let out_of_memory = iteration_stats(&mut csv_out, iteration, &eg, false, start_time);
        if out_of_memory {
            dbg!("reached memory limit!");
            break;
        }
    }

    dbg!(extract::<_, _, AstSizeNoLet>(&i1, &eg));
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
#[tracing::instrument(level = "trace", skip_all)]
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
        eg.total_number_of_nodes(), // TODO: remove
        // eg.ids().into_iter().map(|c| eg.enodes(c).len()).sum::<usize>(),
        eg.ids().len(),
        start_time.elapsed().as_secs_f64(),
        found).unwrap();
    out_of_memory
}

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let lhs = &args[0];
    let rhs = &args[1];
    let csv_out = &args[2];
    let csv_f = std::fs::File::create(csv_out).unwrap();

    may_trace_assert_reaches(lhs, rhs, csv_f, 60);
}

#[cfg(feature = "trace")]
fn may_trace_assert_reaches<W>(start: &str, goal: &str, mut csv_out: W, steps: usize) where W: std::io::Write {

    use tracing_subscriber;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::prelude::*;
    // use tracing_profile::*;

    println!("<TRACING>");
    /*
    tracing_subscriber::fmt()
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
        .with_max_level(tracing_subscriber::filter::LevelFilter::TRACE)
        .init();
        */
/*
    // let (perfetto, _guard) = PerfettoLayer::new_from_env().unwrap();
    tracing_subscriber::registry()
        .with(PrintTreeLayer::new(PrintTreeConfig {
            attention_above_percent: 25.0,
            relevant_above_percent: 2.5,
            hide_below_percent: 1.0,
            display_unaccounted: true,
            accumulate_events: true
        }))
        // .with(CsvLayer::new("/tmp/slotted-rise-tracing.csv"))
        // .with(perfetto)
        // .with(IttApiLayer::default())
        .init();
*/
    tracing::subscriber::set_global_default(BucketSubscriber::new())
        .expect("setting tracing default failed");

    let span = trace_span!("root");
    span.in_scope(|| {
        assert_reaches(start, goal, csv_out, steps);
    });
}

#[cfg(not(feature = "trace"))]
fn may_trace_assert_reaches<W>(start: &str, goal: &str, mut csv_out: W, steps: usize) where W: std::io::Write {
    assert_reaches(start, goal, csv_out, steps);
}