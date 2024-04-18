use crate::*;

mod rewrite;
pub use rewrite::*;

mod my_cost;
pub use my_cost::*;

mod tst;
pub use tst::*;

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum LetENode {
    Lam(Slot, AppliedId),
    App(AppliedId, AppliedId),
    Var(Slot),
    Let(Slot, AppliedId, AppliedId),
}

impl Language for LetENode {
    fn all_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        let mut out = Vec::new();
        match self {
            LetENode::Lam(x, b) => {
                out.push(x);
                out.extend(b.slots_mut());
            },
            LetENode::App(l, r) => {
                out.extend(l.slots_mut());
                out.extend(r.slots_mut());
            }
            LetENode::Var(x) => {
                out.push(x);
            }
            LetENode::Let(x, t, b) => {
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
            LetENode::Lam(x, b) => {
                out.extend(b.slots_mut().into_iter().filter(|y| *y != x));
            },
            LetENode::App(l, r) => {
                out.extend(l.slots_mut());
                out.extend(r.slots_mut());
            }
            LetENode::Var(x) => {
                out.push(x);
            }
            LetENode::Let(x, t, b) => {
                out.extend(b.slots_mut().into_iter().filter(|y| *y != x));
                out.extend(t.slots_mut());
            }
        }
        out
    }

    fn applied_id_occurences_mut(&mut self) -> Vec<&mut AppliedId> {
        match self {
            LetENode::Lam(_, b) => vec![b],
            LetENode::App(l, r) => vec![l, r],
            LetENode::Var(_) => vec![],
            LetENode::Let(_, t, b) => vec![t, b],
        }
    }
}


use std::fmt::*;

impl Debug for LetENode {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            LetENode::Lam(s, b) => write!(f, "(lam {s:?} {b:?})"),
            LetENode::App(l, r) => write!(f, "(app {l:?} {r:?})"),
            LetENode::Var(s) => write!(f, "{s:?}"),
            LetENode::Let(x, t, b) => write!(f, "(let {x:?} {t:?} {b:?})"),
        }
    }
}
