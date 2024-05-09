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
pub struct Slot(usize);

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct AppliedId {
    pub id: Id,

    // m is always a bijection!
    // m maps the slots from `id` (be it ENode::slots() in a RecExpr, or EGraph::slots(Id) for eclasses) to the slots that we insert into it.
    // m.keys() == id.slots
    pub m: SlotMap,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecExpr<L: Language> {
    pub node_dag: Vec<L>,
}

impl<L: Language> RecExpr<L> {
    pub fn empty() -> Self {
        RecExpr { node_dag: Vec::new() }
    }

    pub fn push(&mut self, l: L) {
        self.node_dag.push(l);
    }

    pub fn head_id(&self) -> Id {
        Id(self.node_dag.len() - 1)
    }

    pub fn extend(&mut self, other: RecExpr<L>) {
        let n = self.node_dag.len();
        let shift = |x: AppliedId| -> AppliedId {
            AppliedId::new(Id(x.id.0 + n), x.m)
        };
        self.node_dag.extend(other.node_dag.iter().map(|x: &L| x.map_applied_ids(shift)));
    }
}

impl AppliedId {
    pub fn new(id: Id, m: SlotMap) -> Self {
        let s = AppliedId { id, m };
        s.check();
        s
    }

    pub fn check(&self) {
        assert!(self.m.is_bijection());
    }

    pub fn apply_slotmap(&self, m: &SlotMap) -> AppliedId {
        assert!(m.keys().is_superset(&self.slots()), "AppliedId::apply_slotmap: The SlotMap doesn't map all free slots!");
        self.apply_slotmap_partial(m)
    }

    pub fn apply_slotmap_partial(&self, m: &SlotMap) -> AppliedId {
        AppliedId::new(
            self.id,
            self.m.compose_partial(m),
        )
    }

    pub fn slots(&self) -> HashSet<Slot> {
        self.m.values()
    }

    // ordered!
    pub fn slots_mut(&mut self) -> Vec<&mut Slot> {
        self.m.values_mut().collect()
    }
}

impl Slot {
    pub fn fresh() -> Self {
        use std::cell::RefCell;

        // We choose ThreadLocal here, so that tests (that run in parallel threads) don't interfere.
        // There were situations, where different Slot-names did affect hashmap ordering, and with that actually changed the output of the algorithm.
        // Using this, all tests should run deterministically.

        thread_local! {
            // starting with 100 might prevent some collisions with shape variables.
            static CTR: RefCell<usize> = RefCell::new(100);
        }

        let u = CTR.with_borrow(|v| *v);

        CTR.with_borrow_mut(|v| *v += 1);

        Slot(u)
    }

    pub fn new(u: usize) -> Slot {
        if u >= 100 {
            panic!("slots >= 100 are used for fresh slots!");
        }
        Slot(u)
    }

    pub fn get(&self) -> usize {
        self.0
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
