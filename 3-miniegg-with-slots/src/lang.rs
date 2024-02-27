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
    pub fn map_ids(&self, f: impl Fn(AppliedId) -> AppliedId) -> ENode {
        match self {
            ENode::Lam(x, i) => ENode::Lam(*x, f(i.clone())),
            ENode::App(i1, i2) => ENode::App(f(i1.clone()), f(i2.clone())),
            ENode::Var(x) => ENode::Var(*x),
        }
    }

    pub fn map_slots(&self, f: impl Fn(Slot) -> Slot) -> ENode {
        match self {
            ENode::Lam(x, i) => ENode::Lam(f(*x), i.map_slots(f)),
            ENode::App(i1, i2) => ENode::App(i1.map_slots(&f), i2.map_slots(f)),
            ENode::Var(x) => ENode::Var(f(*x)),
        }
    }

    // lists all slot occurences in order.
    pub fn slot_occurences(&self) -> Vec<Slot> {
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

    // returns a lossy, normalized version of the ENode, by renaming the Slots to be deterministically ordered by their first usage.
    pub fn shape(&self) -> ENode {
        // TODO can I simplify this using slot_order?
        let slots = self.slot_occurences();

        // maps the old slot name to the new order-based name.
        let mut slotmap: HashMap<Slot, Slot> = HashMap::new();

        for x in slots {
            if !slotmap.contains_key(&x) {
                let n = Slot(slotmap.len());
                slotmap.insert(x, n);
            }
        }

        self.map_slots(|s| slotmap[&s])
    }
}

impl AppliedId {
    pub fn new(id: Id, args: Vec<Slot>) -> Self {
        // every Slot can be used at most once!
        let args_set: HashSet<Slot> = args.iter().copied().collect();
        assert_eq!(args.len(), args_set.len());

        AppliedId { id, args }
    }

    pub fn map_slots(&self, f: impl Fn(Slot) -> Slot) -> AppliedId {
        AppliedId::new(
            self.id,
            self.args.iter().copied().map(f).collect(),
        )
    }
}
