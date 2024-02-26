use crate::*;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id(pub usize);

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
// For each eclass, its slots form an interval [0..n].
pub struct Slot(pub usize);

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct AppliedId {
    pub id: Id,
    pub args: Vec<Slot>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum ENode {
    Lam(Slot, AppliedId),
    App(AppliedId, AppliedId),
    Var(Slot),
}

#[derive(Clone)]
pub struct RecExpr {
    pub node_dag: Vec<ENode>,
}

impl ENode {
    pub fn map_ids(self, f: impl Fn(AppliedId) -> AppliedId) -> ENode {
        match self {
            ENode::Lam(x, i) => ENode::Lam(x, f(i)),
            ENode::App(i1, i2) => ENode::App(f(i1), f(i2)),
            ENode::Var(x) => ENode::Var(x),
        }
    }
}
