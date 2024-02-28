use crate::*;
use std::fmt::*;

impl Debug for Slot {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "s{}", self.0)
    }
}

impl Debug for Id {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "id{}", self.0)
    }
}

impl Debug for SlotMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "[")?;
        let n = self.len();
        for (i, (x, y)) in self.iter().enumerate() {
            write!(f, "{x:?} -> {y:?}")?;
            if i < n-1 {
                write!(f, ", ")?;
            }
        }
        write!(f, "]")
    }
}

impl Debug for AppliedId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:?}{:?}", self.id, self.m)
    }
}

impl Debug for ENode {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ENode::Lam(s, b) => write!(f, "(lam {s:?} {b:?})"),
            ENode::App(l, r) => write!(f, "(app {l:?} {r:?})"),
            ENode::Var(s) => write!(f, "{s:?}"),
        }
    }
}

