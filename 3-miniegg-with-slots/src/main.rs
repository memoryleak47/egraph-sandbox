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

fn main() {}
