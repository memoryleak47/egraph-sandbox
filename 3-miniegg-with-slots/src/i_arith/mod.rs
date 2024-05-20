#![allow(unused)]
#![allow(non_snake_case)]

use crate::*;

pub mod build;

mod tst;
pub use tst::*;

mod rewrite;
pub use rewrite::*;

mod my_cost;
pub use my_cost::*;

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ArithENode {
    // lambda calculus:
    Lam(Slot, AppliedId),
    App(AppliedId, AppliedId),
    Var(Slot),
    Let(Slot, AppliedId, AppliedId),

    // rest:
    Number(u32),
    Symbol(Symbol),
}

impl Language for ArithENode {
    fn all_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        let mut out = Vec::new();
        match self {
            ArithENode::Lam(x, b) => {
                out.push(x);
                out.extend(b.slots_mut());
            }
            ArithENode::App(l, r) => {
                out.extend(l.slots_mut());
                out.extend(r.slots_mut());
            }
            ArithENode::Var(x) => {
                out.push(x);
            }
            ArithENode::Let(x, t, b) => {
                out.push(x);
                out.extend(t.slots_mut());
                out.extend(b.slots_mut());
            }
            ArithENode::Number(_) => {}
            ArithENode::Symbol(_) => {}
        }
        out
    }

    fn public_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        let mut out = Vec::new();
        match self {
            ArithENode::Lam(x, b) => {
                out.extend(b.slots_mut().into_iter().filter(|y| *y != x));
            }
            ArithENode::App(l, r) => {
                out.extend(l.slots_mut());
                out.extend(r.slots_mut());
            }
            ArithENode::Var(x) => {
                out.push(x);
            }
            ArithENode::Let(x, t, b) => {
                out.extend(b.slots_mut().into_iter().filter(|y| *y != x));
                out.extend(t.slots_mut());
            }
            ArithENode::Number(_) => {}
            ArithENode::Symbol(_) => {}
        }
        out
    }

    fn applied_id_occurences_mut(&mut self) -> Vec<&mut AppliedId> {
        match self {
            ArithENode::Lam(_, b) => vec![b],
            ArithENode::App(l, r) => vec![l, r],
            ArithENode::Var(_) => vec![],
            ArithENode::Let(_, t, b) => vec![t, b],
            ArithENode::Number(_) => vec![],
            ArithENode::Symbol(_) => vec![],
        }
    }
}


use std::fmt::*;

impl Debug for ArithENode {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ArithENode::Lam(s, b) => write!(f, "(lam {s:?} {b:?})"),
            ArithENode::App(l, r) => write!(f, "(app {l:?} {r:?})"),
            ArithENode::Var(s) => write!(f, "{s:?}"),
            ArithENode::Let(x, t, b) => write!(f, "(let {x:?} {t:?} {b:?})"),
            ArithENode::Number(i) => write!(f, "{i}"),
            ArithENode::Symbol(i) => write!(f, "symb{i:?}"),
        }
    }
}
