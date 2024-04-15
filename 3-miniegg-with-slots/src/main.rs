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

use std::hash::Hash;
use std::fmt::Debug;
pub type HashMap<K, V> = fnv::FnvHashMap<K, V>;
pub type HashSet<T> = fnv::FnvHashSet<T>;

fn main() {
    let s = app(app(add(), num(2)), num(2));
    check_simplify::<LetExpr>(&s, 22);
}
