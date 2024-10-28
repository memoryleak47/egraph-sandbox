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
    let mut csv_f = std::fs::File::create(csv_out).unwrap();

    if cfg!(feature = "tracing") {
        use tracing_subscriber;
        use tracing_subscriber::layer::SubscriberExt;
        use tracing_subscriber::prelude::*;
        use tracing_profile::*;

        println!("<TRACING>");
        /*
        tracing_subscriber::fmt()
            .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
            .with_max_level(tracing_subscriber::filter::LevelFilter::TRACE)
            .init();
            */

        tracing_subscriber::registry()
            .with(PrintTreeLayer::default())
            // .with(CsvLayer::new("/tmp/slotted-rise-tracing.csv"))
            .init();

        let span = trace_span!("root");
        span.in_scope(|| {
            assert_reaches(lhs, rhs, csv_f, 60);
        });
/*
        DEPRECATED (tracing-timing):

        use tracing::{dispatcher, Dispatch};
        use tracing_timing::{Builder, Histogram};

        let subscriber = Builder::default().build(|| Histogram::new_with_max(1_000_000, 2).unwrap());
        let sid = subscriber.downcaster();
        let dispatcher = Dispatch::new(subscriber);
        dispatcher::with_default(&dispatcher, || {
            assert_reaches(lhs, rhs, csv_f, 60);
        });

        sid.downcast(&dispatcher).unwrap().with_histograms(|hs| {
            for (span_group, hs) in hs {
                for (event_group, h) in hs {
                    println!("span: {}, event: {}", span_group, event_group);

                    println!(
                        "mean: {:.1}µs, p50: {}µs, p90: {}µs, p99: {}µs, p999: {}µs, max: {}µs",
                        h.mean() / 1000.0,
                        h.value_at_quantile(0.5) / 1_000,
                        h.value_at_quantile(0.9) / 1_000,
                        h.value_at_quantile(0.99) / 1_000,
                        h.value_at_quantile(0.999) / 1_000,
                        h.max() / 1_000,
                    );
                    for v in break_once(
                        h.iter_linear(25_000).skip_while(|v| v.quantile() < 0.01),
                        |v| v.quantile() > 0.95,
                    ) {
                        println!(
                            "{:4}µs | {:40} | {:4.1}th %-ile",
                            (v.value_iterated_to() + 1) / 1_000,
                            "*".repeat(
                                (v.count_since_last_iteration() as f64 * 40.0 / h.len() as f64).ceil() as usize
                            ),
                            v.percentile(),
                        );
                    }
                }
            }
        });
        */
    } else {
        assert_reaches(lhs, rhs, csv_f, 60);
    }
}

#[cfg(feature = "tracing")]
fn break_once<I, F>(it: I, mut f: F) -> impl Iterator<Item = I::Item>
where
    I: IntoIterator,
    F: FnMut(&I::Item) -> bool,
{
    let mut got_true = false;
    it.into_iter().take_while(move |i| {
        if got_true {
            // we've already yielded when f was true
            return false;
        }
        if f(i) {
            // this must be the first time f returns true
            // we should yield i, and then no more
            got_true = true;
        }
        // f returned false, so we should keep yielding
        true
    })
}