
mod lang;
mod rewrite;
mod my_cost;
mod my_cost2;
pub use lang::*;
pub use rewrite::*;
pub use my_cost::*;
pub use my_cost2::*;

mod dblang;
mod dbanalysis;
mod dbrewrite;
pub use dblang::*;
pub use dbanalysis::*;
pub use dbrewrite::*;

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

fn assert_reaches<W>(start: &str, goal: &str, binding: &str, csv_out: W, steps: usize) where W: std::io::Write {
     match binding {
        "slot" => {
            let start = RecExpr::parse(start).unwrap();
            let goal = RecExpr::parse(goal).unwrap();
            let rules = rise_rules(RiseSubstMethod::SmallStep);
            assert_reaches_common(start, goal, rules, csv_out, steps);
        }
        "de-bruijn" => {
            let start = RecExpr::parse(start).unwrap();
            let goal = RecExpr::parse(goal).unwrap();
            let rules = db_rise_rules();
            assert_reaches_common(to_db(start), to_db(goal), rules, csv_out, steps);
        }
        _ => panic!("did expect binding '{}'", binding)
    };
}

struct Iteration {
    physical_mem: usize,
    virtual_mem: usize,
    egraph_nodes: usize,
    egraph_classes: usize,
    total_time: f64,
    found: bool
}

fn assert_reaches_common<W, L, N>(
    start: RecExpr<L>, goal: RecExpr<L>, rules: Vec<Rewrite<L, N>>,
    mut csv_out: W, steps: usize)
    where W: std::io::Write, L: Language, N: Analysis<L>
{
    let mut eg = EGraph::new();
    let i1 = eg.add_expr(start.clone());
    for it_number in 0..steps {
        let start_time = Instant::now();

        let check_span = trace_span!("check").entered();
        dbg!(it_number, eg.total_number_of_nodes());
        let memory = memory_stats().expect("could not get current memory usage");
        let out_of_memory = memory.virtual_mem > 4_000_000_000;
        if out_of_memory {
            dbg!("reached memory limit!");
        }
        let mut it = Iteration {
            physical_mem: memory.physical_mem,
            virtual_mem: memory.virtual_mem,
            egraph_nodes: eg.total_number_of_nodes(),
            egraph_classes: eg.ids().len(),
            total_time: 0.0,
            found: false
        };

        if let Some(i2) = lookup_rec_expr(&goal, &eg) {
            if eg.eq(&i1, &i2) {
                #[cfg(feature = "explanations")]
                println!("{}", eg.explain_equivalence(start, goal).to_string(&eg));
                it.found = true;
            }
        }
        let stop = it.found || out_of_memory;
        check_span.exit();

        if !stop {
            apply_rewrites(&mut eg, &rules);
        }

        it.total_time = start_time.elapsed().as_secs_f64();
        writeln!(csv_out, "{}, {}, {}, {}, {}, {}, {}",
            it_number,
            it.physical_mem,
            it.virtual_mem,
            it.egraph_nodes,
            it.egraph_classes,
            it.total_time,
            it.found
        ).unwrap();

        if stop {
            return;
        }
    }

    // dbg!(extract::<_, _, AstSizeNoLet>(&i1, &eg));
    dbg!(&goal);
    assert!(false);
}

fn to_db(e: RecExpr<Rise>) -> RecExpr<DBRise> {
    fn rec(expr: RecExpr<Rise>, bound: &[Slot]) -> RecExpr<DBRise> {
        match expr.node {
            Rise::Number(n) => RecExpr { node: DBRise::Number(n as i32), children: vec![] },
            Rise::Symbol(s) => RecExpr { node: DBRise::Symbol(s), children: vec![] },
            Rise::Var(x) => {
                let pos = bound.iter().position(|&s| s == x)
                    .unwrap_or_else(|| panic!("{} not bound", x));
                RecExpr { node: DBRise::Var(Index(pos as u32)), children: vec![] }
            },
            Rise::Lam(x, _) => {
                let mut bound2 = vec![x];
                bound2.extend_from_slice(&bound[..]);
                let children = expr.children.into_iter().map(|c| rec(c, &bound2[..])).collect();
                RecExpr { node: DBRise::Lam(AppliedId::null()), children }
            }
            Rise::App(_, _) => {
                let children = expr.children.into_iter().map(|c| rec(c, &bound[..])).collect();
                RecExpr { node: DBRise::App(AppliedId::null(), AppliedId::null()), children }
            }
            Rise::Let(_, _, _) => unimplemented!(),
        }
    }

    rec(e, &[])
}

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let lhs = &args[0];
    let rhs = &args[1];
    let binding = &args[2];
    let csv_out = &args[3];
    let csv_f = std::fs::File::create(csv_out).unwrap();

    may_trace_assert_reaches(lhs, rhs, binding, csv_f, 60);
}

#[cfg(feature = "trace")]
fn may_trace_assert_reaches<W>(start: &str, goal: &str, binding: &str, csv_out: W, steps: usize) where W: std::io::Write {

    use tracing_subscriber;
    // use tracing_subscriber::layer::SubscriberExt;
    // use tracing_subscriber::prelude::*;
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
        assert_reaches(start, goal, binding, csv_out, steps);
    });
    trace!(name: "display", ""); // trigger display of stats
}

#[cfg(not(feature = "trace"))]
fn may_trace_assert_reaches<W>(start: &str, goal: &str, binding: &str, csv_out: W, steps: usize) where W: std::io::Write {
    assert_reaches(start, goal, binding, csv_out, steps);
}