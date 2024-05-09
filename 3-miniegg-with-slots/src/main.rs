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

mod pattern;
use pattern::*;

use std::hash::Hash;
use std::fmt::Debug;
pub type HashMap<K, V> = fnv::FnvHashMap<K, V>;
pub type HashSet<T> = fnv::FnvHashSet<T>;

fn main() {
    let p = "(lam f (lam arg arg))";
    let s = app(y(), String::from(p));

    let out = simplify::<LetReal>(&s);
    assert_alpha_eq(&out, "(lam x x)");
}
