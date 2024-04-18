use lamcalc::*;

mod types;
use types::*;

mod lang;
use lang::*;

mod i_lambda;
pub use i_lambda::*;

mod i_let;
pub use i_let::*;

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
