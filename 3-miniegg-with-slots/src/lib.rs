#![allow(unused_imports)]

use symbol_table::GlobalSymbol as Symbol;
use lamcalc::*;

mod types;
pub use types::*;

mod lang;
pub use lang::*;

mod i_lambda;
pub use i_lambda::*;

mod i_let;
pub use i_let::*;

mod i_rise;
pub use i_rise::*;

mod i_arith;
pub use i_arith::*;

mod i_spores;
pub use i_spores::*;

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

// Whether to enable invariant-checks.
const CHECKS: bool = false;

use std::hash::Hash;
use std::fmt::Debug;

pub type HashMap<K, V> = fnv::FnvHashMap<K, V>;
pub type HashSet<T> = fnv::FnvHashSet<T>;
