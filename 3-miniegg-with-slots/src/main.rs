use lamcalc::*;

mod types;
use types::*;

mod lang;
use lang::*;

mod lambda;
use lambda::*;

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

mod small_step;
use small_step::*;

mod tst;
use tst::*;

use std::hash::Hash;
use std::fmt::Debug;
pub type HashMap<K, V> = fnv::FnvHashMap<K, V>;
pub type HashSet<T> = fnv::FnvHashSet<T>;

fn main() {
    let p = "(lam x (lam y (app (lam z (app x z)) y)))";
    let p2 = "(lam x (lam y (app x y)))";
    check_simplify::<Expr<SmallStep>>(p, 10);
}
