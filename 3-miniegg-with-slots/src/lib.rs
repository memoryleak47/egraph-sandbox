#![allow(unused_imports)]

use symbol_table::GlobalSymbol as Symbol;
use lamcalc::*;

use std::hash::Hash;
use std::fmt::Debug;
use std::error::Error;

pub type HashMap<K, V> = fnv::FnvHashMap<K, V>;
pub type HashSet<T> = fnv::FnvHashSet<T>;

// Whether to enable invariant-checks.
const CHECKS: bool = true;

mod types;
pub use types::*;

mod parse;
pub use parse::*;

mod lang;
pub use lang::*;

mod i_lambda;
pub use i_lambda::*;

mod i_let;
pub use i_let::*;

mod i_rise;
pub use i_rise::*;

mod tst;
pub use tst::*;

mod i_array;
pub use i_array::*;

mod i_arith;
pub use i_arith::*;

mod i_spores;
pub use i_spores::*;

mod i_symbol;
pub use i_symbol::*;

mod slotmap;
pub use slotmap::*;

mod debug;

mod egraph;
pub use egraph::*;

mod extract;
pub use extract::*;

mod pattern;
pub use pattern::*;

mod group;
use group::*;
