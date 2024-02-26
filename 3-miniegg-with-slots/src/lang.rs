use crate::*;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id(pub usize);

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ENode {
    Lam(String, Id),
    App(Id, Id),
    Var(String),
}

#[derive(Clone)]
pub struct RecExpr {
    pub node_dag: Vec<ENode>,
}

impl ENode {
    pub fn map_ids(self, f: impl Fn(Id) -> Id) -> ENode {
        match self {
            ENode::Lam(x, i) => ENode::Lam(x, f(i)),
            ENode::App(i1, i2) => ENode::App(f(i1), f(i2)),
            ENode::Var(x) => ENode::Var(x),
        }
    }
}
