use crate::*;

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id(pub usize);

pub type FreeSlot = usize; // These form an interval [0..N].
pub type RedundantSlot = usize; // These form a different interval [0..N].

// TODO maybe have a version of AppliedId with only SlotKind::Free for user-facing stuff?
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

    pub fn map_slot_kinds(&self, f: impl Fn(SlotKind) -> SlotKind) -> ENode {
        match self {
            ENode::Lam(r) => ENode::Lam(r.map_slot_kinds(f)),
            ENode::App(l, r) => ENode::App(l.map_slot_kinds(&f), r.map_slot_kinds(f)),
            ENode::Var => ENode::Var,
        }
    }

    // sorts the redundant slots to be ordered by usage.
    pub fn with_sorted_redundants(&self) -> ENode {
        let mut redundants = HashMap::new();
        for x in self.slot_kind_occurences() {
            if let SlotKind::Redundant(i) = x {
                if !redundants.contains_key(&i) {
                    redundants.insert(i, redundants.len());
                }
            }
        }

        self.map_slot_kinds(|x| {
            match x {
                SlotKind::Redundant(i) => SlotKind::Redundant(redundants[&i]),
                y => y,
            }
        })
    }
}

impl AppliedId {
    fn map_slot_kinds(&self, f: impl Fn(SlotKind) -> SlotKind) -> AppliedId {
        AppliedId {
            id: self.id,
            args: self.args.iter().cloned().map(f).collect(),
        }
    }
}
