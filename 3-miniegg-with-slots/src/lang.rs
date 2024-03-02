use crate::*;

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id(pub usize);

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
// For each eclass, its slots form an interval [0..n].
// An ENode contains three kinds of slots:
// - free / exposed
// - lambda
// - internal (not really part of the ENode API, it's rather the exposed slots of its children)
//
// A slot is "flexible" if it's free or lambda.
pub struct Slot(pub usize);

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct AppliedId {
    pub id: Id,

    // m is always a bijection!
    // m maps the slots from `id` (be it ENode::slots() in a RecExpr, or EGraph::slots(Id) for eclasses) to the slots that we insert into it.
    // m.keys() == id.slots
    pub m: SlotMap,
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum ENode {
    Lam(Slot, AppliedId),
    App(AppliedId, AppliedId),
    Var(Slot),
}

#[derive(Clone, Debug)]
pub struct RecExpr {
    pub node_dag: Vec<ENode>,
}

impl ENode {
    pub fn map_applied_ids(&self, f: impl Fn(AppliedId) -> AppliedId) -> ENode {
        match self {
            ENode::Lam(x, i) => ENode::Lam(*x, f(i.clone())),
            ENode::App(i1, i2) => ENode::App(f(i1.clone()), f(i2.clone())),
            ENode::Var(x) => ENode::Var(*x),
        }
    }

    pub fn apply_slotmap(&self, m: &SlotMap) -> ENode {
        match self {
            ENode::Lam(x, i) => {
                let mut m = m.clone();
                // removing x causes a "missing entry" problem.
                // in order to keep x "unchanged" we insert x -> x.
                m.insert(*x, *x);

                assert!(m.is_bijection()); // if this fails, then probably multiple things point to x now - because someone didn't rename enough stuff.

                ENode::Lam(*x, i.apply_slotmap(&m))
            },
            ENode::App(i1, i2) => ENode::App(i1.apply_slotmap(&m), i2.apply_slotmap(&m)),
            ENode::Var(x) => ENode::Var(m[*x]),
        }
    }

    pub fn slot_occurences(&self) -> Vec<Slot> {
        let mut v = Vec::new();
        match self {
            ENode::Lam(s, r) => {
                v.extend(r.m.values().into_iter().filter(|x| x != s));
            },
            ENode::App(l, r) => {
                v.extend(l.m.values());
                v.extend(r.m.values());
            }
            ENode::Var(s) => {
                v.push(*s);
            },
        };

        v
    }

    pub fn slot_order(&self) -> Vec<Slot> { firsts(self.slot_occurences()) }
    pub fn slots(&self) -> HashSet<Slot> { as_set(self.slot_occurences()) }

    // self == other, while being blind for the ENode::Lam(_) slot-name and also being blind for free slots not contained in "exposed".
    // free slots not contained in exposed are conceptually "redundant slots" within some EClass.
    pub fn equal_within_exposed(&self, other: &ENode, exposed: &HashSet<Slot>) -> bool {
        todo!()
    }
}

// sorts as_set(v) by their first usage in v.
pub fn firsts(v: Vec<Slot>) -> Vec<Slot> {
    let mut out = Vec::new();
    for x in v {
        if !out.contains(&x) {
            out.push(x);
        }
    }
    out
}

pub fn as_set(v: Vec<Slot>) -> HashSet<Slot> {
    v.into_iter().collect()
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

    pub fn slots(&self) -> HashSet<Slot> {
        self.m.values()
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


#[test]
fn test_apply_slotmap() {
    let s = Slot;

    let in_slotmap = SlotMap::from_pairs(&[(s(0), s(10)), (s(1), s(3))]);
    let in_enode = ENode::Lam(s(3), AppliedId::new(Id(12), in_slotmap));

    let slotmap = SlotMap::from_pairs(&[(s(10), s(100))]);
    let real = in_enode.apply_slotmap(&slotmap);

    let expected_slotmap = SlotMap::from_pairs(&[(s(0), s(100)), (s(1), s(3))]);
    let expected = ENode::Lam(s(3), AppliedId::new(Id(12), expected_slotmap));
    assert_eq!(real, expected);
}
