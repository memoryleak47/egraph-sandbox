use crate::*;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id(pub usize);

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
// For each eclass, its slots form an interval [0..n].
pub struct Slot(pub usize);

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct AppliedId {
    pub id: Id,

    // m is always a bijection!
    // m maps the slots from `id` (be it ENode::slots() in a RecExpr, or EGraph::slots(Id) for eclasses) to the slots that we insert into it.
    // m.keys() == id.slots
    pub m: SlotMap,
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

impl RecExpr {
    pub fn new() -> Self {
        RecExpr { node_dag: Vec::new() }
    }

    pub fn push(&mut self, t: ENode) -> AppliedId {
        self.node_dag.push(t);
        todo!()
    }
}

impl ENode {
    pub fn map_ids(&self, f: impl Fn(AppliedId) -> AppliedId) -> ENode {
        match self {
            ENode::Lam(x, i) => ENode::Lam(*x, f(i.clone())),
            ENode::App(i1, i2) => ENode::App(f(i1.clone()), f(i2.clone())),
            ENode::Var(x) => ENode::Var(*x),
        }
    }

    pub fn apply_slotmap(&self, m: &SlotMap) -> ENode {
        match self {
            ENode::Lam(x, i) => ENode::Lam(m[*x], i.apply_slotmap(&m)),
            ENode::App(i1, i2) => ENode::App(i1.apply_slotmap(&m), i2.apply_slotmap(&m)),
            ENode::Var(x) => ENode::Var(m[*x]),
        }
    }

    // lists all slot occurences in order.
    pub fn slot_occurences(&self) -> Vec<Slot> {
        let mut slotlist: Vec<Slot> = Vec::new();

        match self {
            ENode::Lam(s, r) => {
                slotlist.push(*s);
                slotlist.extend(r.m.values());
            },
            ENode::App(l, r) => {
                slotlist.extend(l.m.values());
                slotlist.extend(r.m.values());
            }
            ENode::Var(s) => {
                slotlist.push(*s);
            },
        };

        slotlist
    }

    // equivalent to slot_occurences, but with duplicates removed.
    pub fn slot_order(&self) -> Vec<Slot> {
        let mut out = Vec::new();
        let mut done = HashSet::new();
        for x in self.slot_occurences() {
            if !done.contains(&x) {
                done.insert(x);
                out.push(x);
            }
        }

        out
    }

    pub fn slots(&self) -> HashSet<Slot> {
        self.slot_occurences().into_iter().collect()
    }

    // returns a lossy, normalized version of the ENode, by renaming the Slots to be deterministically ordered by their first usage.
    pub fn shape(&self) -> ENode {
        let slots = self.slot_order();

        // maps the old slot name to the new order-based name.
        let mut slotmap = SlotMap::new();

        for x in slots {
            let n = Slot(slotmap.len());
            slotmap.insert(x, n);
        }

        self.apply_slotmap(&slotmap)
    }
}

impl AppliedId {
    pub fn new(id: Id, m: SlotMap) -> Self {
        assert!(m.is_bijection());

        AppliedId { id, m }
    }

    pub fn apply_slotmap(&self, m: &SlotMap) -> AppliedId {
        AppliedId::new(
            self.id,
            self.m.compose(m),
        )
    }
}

impl Slot {
    pub fn fresh() -> Self {
        use std::sync::atomic::*;

        static CTR: AtomicUsize = AtomicUsize::new(0);
        let u = CTR.fetch_add(1, Ordering::SeqCst);
        Slot(u)
    }
}
