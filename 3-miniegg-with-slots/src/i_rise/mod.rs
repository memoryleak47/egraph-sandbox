use crate::*;

mod rewrite;
pub use rewrite::*;

mod my_cost;
pub use my_cost::*;

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiseENode {
    Lam(Slot, AppliedId),
    App(AppliedId, AppliedId),
    Var(Slot),
    Let(Slot, AppliedId, AppliedId),
}

impl Language for RiseENode {
    fn all_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        let mut out = Vec::new();
        match self {
            RiseENode::Lam(x, b) => {
                out.push(x);
                out.extend(b.slots_mut());
            },
            RiseENode::App(l, r) => {
                out.extend(l.slots_mut());
                out.extend(r.slots_mut());
            }
            RiseENode::Var(x) => {
                out.push(x);
            }
            RiseENode::Let(x, t, b) => {
                out.push(x);
                out.extend(t.slots_mut());
                out.extend(b.slots_mut());
            }
        }
        out
    }

    fn public_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        let mut out = Vec::new();
        match self {
            RiseENode::Lam(x, b) => {
                out.extend(b.slots_mut().into_iter().filter(|y| *y != x));
            },
            RiseENode::App(l, r) => {
                out.extend(l.slots_mut());
                out.extend(r.slots_mut());
            }
            RiseENode::Var(x) => {
                out.push(x);
            }
            RiseENode::Let(x, t, b) => {
                out.extend(b.slots_mut().into_iter().filter(|y| *y != x));
                out.extend(t.slots_mut());
            }
        }
        out
    }

    fn applied_id_occurences_mut(&mut self) -> Vec<&mut AppliedId> {
        match self {
            RiseENode::Lam(_, b) => vec![b],
            RiseENode::App(l, r) => vec![l, r],
            RiseENode::Var(_) => vec![],
            RiseENode::Let(_, t, b) => vec![t, b],
        }
    }
}


use std::fmt::*;

impl Debug for RiseENode {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            RiseENode::Lam(s, b) => write!(f, "(lam {s:?} {b:?})"),
            RiseENode::App(l, r) => write!(f, "(app {l:?} {r:?})"),
            RiseENode::Var(s) => write!(f, "{s:?}"),
            RiseENode::Let(x, t, b) => write!(f, "(let {x:?} {t:?} {b:?})"),
        }
    }
}
