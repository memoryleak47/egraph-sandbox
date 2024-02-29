use crate::*;

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id(pub usize);

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
// For each eclass, its slots form an interval [0..n].
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

impl RecExpr {
    pub fn new() -> Self {
        RecExpr { node_dag: Vec::new() }
    }
}

impl ENode {
    pub fn map_applied_ids(&self, f: impl Fn(AppliedId) -> AppliedId) -> ENode {
        match self {
            ENode::Lam(x, i) => ENode::Lam(*x, f(i.clone())),
            ENode::App(i1, i2) => ENode::App(f(i1.clone()), f(i2.clone())),
            ENode::Var(x) => ENode::Var(*x),
        }
    }

    pub fn apply_slotmap_including_lam(&self, m: &SlotMap) -> ENode {
        match self {
            ENode::Lam(x, i) => ENode::Lam(m[*x], i.apply_slotmap(&m)),
            ENode::App(i1, i2) => ENode::App(i1.apply_slotmap(&m), i2.apply_slotmap(&m)),
            ENode::Var(x) => ENode::Var(m[*x]),
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

    pub fn slot_order(&self) -> Vec<Slot> {
        let mut out = Vec::new();
        let mut done = HashSet::new();
        for x in slot_occurences(self) {
            if !done.contains(&x) {
                done.insert(x);
                out.push(x);
            }
        }

        return out;

        // lists all slot occurences in order.
        fn slot_occurences(n: &ENode) -> Vec<Slot> {
            let mut slotlist: Vec<Slot> = Vec::new();

            match n {
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
    }

    // different than set(slot_order), as it doesn't contain lambda slots.
    pub fn slots(&self) -> HashSet<Slot> {
        let mut set = HashSet::new();
        match self {
            ENode::Lam(s, r) => {
                set.extend(r.m.values().into_iter().filter(|x| x != s));
            },
            ENode::App(l, r) => {
                set.extend(l.m.values());
                set.extend(r.m.values());
            }
            ENode::Var(s) => {
                set.insert(*s);
            },
        };

        set
    }

    // returns a lossy, normalized version of the ENode, by renaming the Slots to be deterministically ordered by their first usage.
    // shape() will later be used as a normalized ENode stored in the hashcons.
    pub fn shape(&self) -> ENode {
        let slots = self.slot_order();

        // maps the old slot name to the new order-based name.
        let mut slotmap = SlotMap::new();

        for x in slots {
            let n = Slot(slotmap.len());
            slotmap.insert(x, n);
        }

        self.apply_slotmap_including_lam(&slotmap)
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

    let expected_slotmap = SlotMap::from_pairs(&[(s(0), s(100))]);
    let expected = ENode::Lam(s(3), AppliedId::new(Id(12), expected_slotmap));
    assert_eq!(real, expected);
}
