use lamcalc::*;

mod types;
use types::*;

mod lang;
use lang::*;

mod i_lambda;
use i_lambda::*;

mod i_let;
use i_let::*;

mod syntax;
use syntax::*;

mod slotmap;
use slotmap::*;

mod debug;

mod egraph;
use egraph::*;

mod extract;
use extract::*;

mod tst;
use tst::*;

mod pattern;
use pattern::*;

use std::hash::Hash;
use std::fmt::Debug;
pub type HashMap<K, V> = fnv::FnvHashMap<K, V>;
pub type HashSet<T> = fnv::FnvHashSet<T>;

fn main() {
    // let s = app(app(add(), num(2)), num(2));
    let s = "(app (app (app (lam b (lam b (app b (lam z b)))) (app (lam z z) (lam c c))) (lam y (app y (app y (lam b b))))) (lam c (lam y (app (lam b (app b (lam y (app (lam c c) c)))) (lam c (app (lam c (lam x x)) (lam a (lam a (app a (lam y a))))))))))";
    let s = format!("(app {} {})", s, s);

    dbg!(simplify::<LetExpr>(&s, 22));
}
