use crate::*;

mod rewrite;
pub use rewrite::*;

mod tst;

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum LetENode {
    Lam(Slot, AppliedId),
    App(AppliedId, AppliedId),
    Var(Slot),
    Let(Slot, AppliedId, AppliedId),
}

impl Language for LetENode {
    fn discr(&self) -> u32 {
        match self {
            LetENode::Lam(_, _) => 0,
            LetENode::App(_, _) => 1,
            LetENode::Var(_) => 2,
            LetENode::Let(..) => 3,
        }
    }

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
