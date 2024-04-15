use crate::*;

mod rewrite;
pub use rewrite::*;

mod subst;
pub use subst::*;

mod small_step;
pub use small_step::*;

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ENode {
    Lam(Slot, AppliedId),
    App(AppliedId, AppliedId),
    Var(Slot),
}

impl Language for ENode {
    fn discr(&self) -> u32 {
        match self {
            ENode::Lam(_, _) => 0,
            ENode::App(_, _) => 1,
            ENode::Var(_) => 2,
        }
    }

    fn all_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        let mut out = Vec::new();
        match self {
            ENode::Lam(x, b) => {
                out.push(x);
                out.extend(b.slots_mut());
            },
            ENode::App(l, r) => {
                out.extend(l.slots_mut());
                out.extend(r.slots_mut());
            }
            ENode::Var(x) => {
                out.push(x);
            }
        }
        out
    }

    fn public_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        let mut out = Vec::new();
        match self {
            ENode::Lam(x, b) => {
                out.extend(b.slots_mut().into_iter().filter(|y| *y != x));
            },
            ENode::App(l, r) => {
                out.extend(l.slots_mut());
                out.extend(r.slots_mut());
            }
            ENode::Var(x) => {
                out.push(x);
            }
        }
        out
    }

    fn applied_id_occurences_mut(&mut self) -> Vec<&mut AppliedId> {
        match self {
            ENode::Lam(_, b) => vec![b],
            ENode::App(l, r) => vec![l, r],
            ENode::Var(_) => vec![],
        }
    }
}
