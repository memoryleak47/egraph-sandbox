mod lang;
use lang::*;

mod shape;
use shape::*;

mod ast;
use ast::*;

mod syntax;
use syntax::*;

mod slotmap;
use slotmap::*;

mod debug;

mod egraph;
use egraph::*;

mod extract;
use extract::*;

mod rewrite;
use rewrite::*;

mod subst;
use subst::*;

mod tst;

use std::collections::{BTreeMap, BTreeSet};
// TODO maybe choose an actual HashMap that is deterministic.
// Tree Maps are logarithmic in most operations, whereas hashmaps are typically O(1).
pub type HashMap<K, V> = BTreeMap<K, V>;
pub type HashSet<T> = BTreeSet<T>;

// current example:

fn y() -> String {
    let a = format!("(lam x (app f (app x x)))");

    format!("(lam f (app {a} {a}))")
}

fn app(x: String, y: String) -> String {
    format!("(app {x} {y})")
}

fn main() {
    let inf_impl = format!("(lam inf (lam arg (app inf arg)))");
    let s = app(y(), inf_impl);

    let re = RecExpr::parse(&s);
    let mut eg = EGraph::new();
    let i = eg.add_expr(re.clone());

    eg.inv();
    rewrite_step(&mut eg);
    eg.inv();

    let re = extract(i, &eg);
    let s = re.to_string();
    println!("{}", s);
}
