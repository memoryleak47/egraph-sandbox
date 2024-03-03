use crate::*;
use std::fmt::*;

impl Debug for Id {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "id{}", self.0)
    }
}

impl Debug for AppliedId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:?}{:?}", self.id, self.args)
    }
}

impl Debug for ENode {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ENode::Lam(b) => write!(f, "(lam {b:?})"),
            ENode::App(l, r) => write!(f, "(app {l:?} {r:?})"),
            ENode::Var => write!(f, "<var>"),
        }
    }
}

