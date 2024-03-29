use lamcalc::*;

mod lang;
use lang::*;

mod shape;
use shape::*;

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

pub type HashMap<K, V> = fnv::FnvHashMap<K, V>;
pub type HashSet<T> = fnv::FnvHashSet<T>;

fn main() {
    let p = "(lam x (lam z (app (lam y z) x)))";
    check_simplify::<Expr<SmallStep>>(p, 10);
}
