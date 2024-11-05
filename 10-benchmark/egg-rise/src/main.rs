mod rise;
mod rules;
mod alpha_equiv;
mod dbrise;
mod dbrules;

enum WithExpansion { Yes, No }

use std::fs::File;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let lhs = &*args[0];
    let rhs = &*args[1];
    let binding = &*args[2];
    let csv_out = &args[3];
    let csv_f = File::create(csv_out).unwrap();

    let rules = vec!["beta", "eta", "map-fusion", "map-fission"];

    let bench = |start, goal, rules| {
        bench_prove_equiv(start, goal, rules, binding, csv_f);
    };

    bench(lhs, rhs, &rules)
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

use memory_stats::memory_stats;

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

fn bench_prove_equiv(start_s: &str, goal_s: &str, rule_names: &[&str], binding: &str, csv_out: File) {
    println!();
    println!("-------");
    println!("- lhs:         {}", start_s);
    println!("- rhs:         {}", goal_s);
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

            prove_equiv_aux(start, goal, rules(&*rule_names), csv_out)
        },
        "de-bruijn" => {
            if rule_names.contains(&"beta") {
                rule_names.extend([
                    "sig-unused", "sig-lam", "sig-app", "sig-var",
                    "phi-unused", "phi-lam", "phi-app", "phi-var"
                ]);
            }

            to_db_prove_equiv_aux(start, goal, dbrules(&*rule_names), csv_out)
        },
        _ => panic!("did not expect {}", binding)
    }

    println!();
}

fn prove_equiv_aux(start: RecExpr<Rise>, goal: RecExpr<Rise>, rules: Vec<Rewrite<Rise, RiseAnalysis>>, csv_out: File) {
    let goal = expr_to_alpha_equiv_pattern(goal);
    let goals: Vec<Pattern<Rise>> = vec![goal];
    common_prove_equiv_aux(&start, goals, rules, csv_out);
    // count_alpha_equiv(&mut runner.egraph);
    // runner.egraph.dot().to_svg(format!("/tmp/{}.svg", name)).unwrap();
}

fn db_prove_equiv_aux(start: RecExpr<DBRise>, goal: RecExpr<DBRise>, rules: Vec<Rewrite<DBRise, DBRiseAnalysis>>, csv_out: File) {
   let goals: Vec<Pattern<DBRise>> = vec![goal.as_ref().into()];
   common_prove_equiv_aux(&start, goals, rules, csv_out);
}

fn common_prove_equiv_aux<L, A>(start: &RecExpr<L>, goals: Vec<Pattern<L>>, rules: Vec<Rewrite<L, A>>, mut csv_out: File)
    where A: egg::Analysis<L> + std::default::Default + 'static,
          L: egg::Language + std::fmt::Display + 'static,
{
    let mut runner = Runner::default();
    let id = runner.egraph.add_expr(start);
    runner.roots.push(id);

    let goals2 = goals.clone();
    let mut csv_out2 = csv_out.try_clone().unwrap();
    runner = runner
        .with_scheduler(SimpleScheduler)
        .with_node_limit(100_000_000)
        .with_iter_limit(500)
        .with_time_limit(std::time::Duration::from_secs(60*60)) // 4mn
        .with_hook(move |r| {
            let mut out_of_memory = false;
            if let Some(it) = r.iterations.last() {
                out_of_memory = iteration_stats(&r.egraph, it, r.iterations.len() - 1, &mut csv_out2);
            }

            if goals2.iter().all(|g| g.search_eclass(&r.egraph, id).is_some()) {
                Err("Done".into())
            } else if out_of_memory {
                Err("Out of Memory".into())
            } else {
                Ok(())
            }
        }).run(&rules);


    iteration_stats(&runner.egraph, runner.iterations.last().unwrap(), runner.iterations.len() - 1, &mut csv_out);
    runner.print_report();
    let rules = runner.iterations.iter().map(|i|
        i.applied.iter().map(|(_, n)| n).sum::<usize>()).sum::<usize>();
    println!("applied rules: {}", rules);
    runner.iterations.iter().for_each(|i| println!("{:?}", i));
    runner.egraph.check_goals(id, &goals);
}

// iteration number,
// physical memory,
// virtual memory,
// e-graph nodes (hashcons),
// e-graph nodes (real),
// e-graph classes,
// applied rules,
// total time,
// hook time,
// search time,
// apply time,
// rebuild time,
// found
fn iteration_stats<W, L, N>(egraph: &EGraph<L, N>, it: &egg::Iteration<()>, it_number: usize, csv_out: &mut W) -> bool
    where W: std::io::Write,
    N: egg::Analysis<L> + std::default::Default + 'static,
    L: egg::Language + std::fmt::Display + 'static,
{
    dbg!(it_number, egraph.total_number_of_nodes());
    let memory = memory_stats().expect("could not get current memory usage");
    let out_of_memory = memory.virtual_mem > 4_000_000_000;
    let found = match &it.stop_reason {
        Some(egg::StopReason::Other(s)) => s == "Done",
        _ => false,
    };
    writeln!(csv_out, "{}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}",
        it_number,
        memory.physical_mem,
        memory.virtual_mem,
        it.egraph_nodes,
        egraph.total_number_of_nodes(),
        it.egraph_classes,
        it.applied.iter().map(|(_, &n)| n).sum::<usize>(),
        it.total_time,
        it.hook_time,
        it.search_time,
        it.apply_time,
        it.rebuild_time,
        found).unwrap();
    out_of_memory
}

fn to_db_prove_equiv_aux(start: RecExpr<Rise>, goal: RecExpr<Rise>, rules: Vec<Rewrite<DBRise, DBRiseAnalysis>>, csv_out: File) {
    let start_db = to_db(start);
    let goal_db = to_db(goal);
    println!("start (db): {}", start_db);
    println!("goal  (db): {}", goal_db);
    db_prove_equiv_aux(start_db, goal_db, rules, csv_out)
}
