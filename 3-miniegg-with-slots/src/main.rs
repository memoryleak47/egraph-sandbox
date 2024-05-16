use symbol_table::GlobalSymbol as Symbol;
use lamcalc::{Realization, Ast};

mod types;
use types::*;

mod lang;
use lang::*;

mod i_lambda;
use i_lambda::*;

mod i_let;
use i_let::*;

mod i_rise;
use i_rise::*;

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

mod group;
use group::*;

use std::hash::Hash;
use std::fmt::Debug;
pub type HashMap<K, V> = fnv::FnvHashMap<K, V>;
pub type HashSet<T> = fnv::FnvHashSet<T>;

fn main() {
}
