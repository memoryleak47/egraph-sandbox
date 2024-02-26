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

    // returns a lossy, normalized version of the ENode, by renaming the Slots to be deterministically ordered by their first usage.
    pub fn shape(&self) -> ENode {
        // all occurences of all slots, ordered from left to right through the ENode.
        let mut slotlist: Vec<Slot> = Vec::new();
        match self {
            ENode::Lam(s, r) => {
                slotlist.push(*s);
                slotlist.extend(r.clone().args);
            },
            ENode::App(l, r) => {
                slotlist.extend(l.clone().args);
                slotlist.extend(r.clone().args);
            }
            ENode::Var(s) => {
                slotlist.push(*s);
            },
        };

        // maps the old slot name to the new order-based name.
        let mut slotmap: HashMap<Slot, Slot> = HashMap::new();

        for x in slotlist {
            if !slotmap.contains_key(&x) {
                let n = Slot(slotmap.len());
                slotmap.insert(x, n);
            }
        }

        let f = |s: &Slot| slotmap[s];
        let g = |a: &AppliedId| -> AppliedId {
            AppliedId {
                id: a.id,
                args: a.args.iter().map(f).collect(),
            }
        };

        match self {
            ENode::Lam(s, r) => ENode::Lam(f(s), g(r)),
            ENode::App(l, r) => ENode::App(g(l), g(r)),
            ENode::Var(s) => ENode::Var(f(s)),
        }
    }
}
