use crate::*;

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id(pub usize);

pub type FreeSlot = usize; // These form an interval [0..N].
pub type RedundantSlot = usize; // These form a different interval [0..N].

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct AppliedId {
    pub id: Id,
    pub args: Vec<SlotKind>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum SlotKind {
    Free(FreeSlot),
    Lam,
    Redundant(RedundantSlot),
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum ENode {
    Lam(AppliedId),
    App(AppliedId, AppliedId),
    Var, // always uses FreeSlot 0.
}

#[derive(Clone, Debug)]
pub struct RecExpr {
    pub node_dag: Vec<ENode>,
}

impl ENode {
    pub fn slot_kind_occurences(&self) -> Vec<SlotKind> {
        let mut v = Vec::new();
        match self {
            ENode::Lam(r) => {
                v.extend(r.args.clone());
            },
            ENode::App(l, r) => {
                v.extend(l.args.clone());
                v.extend(r.args.clone());
            }
            ENode::Var => {
                v.push(SlotKind::Free(0));
            },
        };

        v
    }
}
