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

pub trait Language: Debug + Clone + Hash + Eq {
    fn discr(&self) -> u32;

    // returns non-deduplicated lists of all occurences of these things, in order.
    fn all_slot_occurences_mut(&mut self) -> Vec<&mut Slot>;
    fn public_slot_occurences_mut(&mut self) -> Vec<&mut Slot>;
    fn applied_id_occurences_mut(&mut self) -> Vec<&mut AppliedId>;


    fn all_slot_occurences(&self) -> Vec<Slot> {
        self.clone().all_slot_occurences_mut().into_iter().map(|x| x.clone()).collect()
    }

    fn public_slot_occurences(&self) -> Vec<Slot> {
        self.clone().public_slot_occurences_mut().into_iter().map(|x| x.clone()).collect()
    }

    fn applied_id_occurences(&self) -> Vec<AppliedId> {
        self.clone().applied_id_occurences_mut().into_iter().map(|x| x.clone()).collect()
    }
}

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct AppliedId {
    pub id: Id,

    // m is always a bijection!
    // m maps the slots from `id` (be it ENode::slots() in a RecExpr, or EGraph::slots(Id) for eclasses) to the slots that we insert into it.
    // m.keys() == id.slots
    pub m: SlotMap,
}

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ENode {
    Lam(Slot, AppliedId),
    App(AppliedId, AppliedId),
    Var(Slot),
}

impl Language for ENode {
    fn discr(&self) -> u32 {
        match self {
            ENode::Lam(_, _) => 0,
            ENode::App(_, _) => 1,
            ENode::Var(_) => 2,
        }
    }

    fn all_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        let mut out = Vec::new();
        match self {
            ENode::Lam(x, b) => {
                out.push(x);
                out.extend(b.slots_mut());
            },
            ENode::App(l, r) => {
                out.extend(l.slots_mut());
                out.extend(r.slots_mut());
            }
            ENode::Var(x) => {
                out.push(x);
            }
        }
        out
    }

    fn public_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        let mut out = Vec::new();
        match self {
            ENode::Lam(x, b) => {
                out.extend(b.slots_mut().into_iter().filter(|y| *y != x));
            },
            ENode::App(l, r) => {
                out.extend(l.slots_mut());
                out.extend(r.slots_mut());
            }
            ENode::Var(x) => {
                out.push(x);
            }
        }
        out
    }

    fn applied_id_occurences_mut(&mut self) -> Vec<&mut AppliedId> {
        match self {
            ENode::Lam(_, b) => vec![b],
            ENode::App(l, r) => vec![l, r],
            ENode::Var(_) => vec![],
        }
    }
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

    pub fn apply_slotmap_partial(&self, m: &SlotMap) -> ENode {
        match self {
            ENode::Lam(x, i) => {
                if m.contains_key(*x) {
                    panic!("apply_slotmap applied in lambda trying to rename its Lam variable.");
                }
                if m.values().contains(x) {
                    panic!("apply_slotmap applied in lambda trying to rename *to* its Lam variable.");
                }

                let mut m = m.clone();
                // removing x causes a "missing entry" problem.
                // in order to keep x "unchanged" we insert x -> x.
                m.insert(*x, *x);

                assert!(m.is_bijection()); // if this fails, then probably multiple things point to x now - because someone didn't rename enough stuff.

                ENode::Lam(*x, i.apply_slotmap_partial(&m))
            },
            ENode::App(i1, i2) => ENode::App(i1.apply_slotmap_partial(&m), i2.apply_slotmap_partial(&m)),
            ENode::Var(x) => ENode::Var(m[*x]),
        }
    }

    #[track_caller]
    pub fn apply_slotmap(&self, m: &SlotMap) -> ENode {
        assert!(m.keys().is_superset(&self.slots()), "ENode::apply_slotmap: The SlotMap doesn't map all free slots!");
        self.apply_slotmap_partial(m)
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

    pub fn ids(&self) -> Vec<Id> {
        match self {
            ENode::App(l, r) => vec![l.id, r.id],
            ENode::Lam(_, b) => vec![b.id],
            ENode::Var(_) => vec![],
        }
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
            // starting with 1 might prevent Slot(0) collisions with Lam variables.
            static CTR: RefCell<usize> = RefCell::new(1);
        }

        let u = CTR.with_borrow(|v| *v);

        CTR.with_borrow_mut(|v| *v += 1);

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
