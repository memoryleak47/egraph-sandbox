use crate::*;

mod add;
pub use add::*;

mod union;
pub use union::*;

#[derive(Clone, Debug)]
pub struct EClass {
    // The set of equivalent ENodes that make up this eclass.
    // for (sh, bij) in nodes; sh.apply_slotmap(bij) represents the actual ENode.
    nodes: HashMap<Shape, Bijection>,

    // All other slots are considered "redundant" (or they have to be qualified by a ENode::Lam).
    // Should not contain Slot(0).
    slots: HashSet<Slot>,
}

// invariants:
// 1. If two ENodes (that are in the EGraph) have equal .shape(), they have to be in the same eclass.
// 2. enode.slots() is always a superset of c.slots, if enode is within the eclass c.
//    if ENode::Lam(si) = enode, then we require i to not be in c.slots.
// 3. AppliedId::m is always a bijection. (eg. c1(s0, s1, s0) is illegal!)
// 4. Slot(0) should not be in EClass::slots of any class.
#[derive(Debug)]
pub struct EGraph {
    // an entry (l, r(sa, sb)) in unionfind corresponds to the equality l(s0, s1, s2) = r(sa, sb), where sa, sb in {s0, s1, s2}.
    // normalizes the eclass.
    // Each Id i that is an output of the unionfind itself has unionfind[i] = (i, identity()).
    unionfind: HashMap<Id, AppliedId>,

    // only ids with unionfind[x].id = x are contained.
    classes: HashMap<Id, EClass>,
}

impl EGraph {
    pub fn new() -> Self {
        EGraph {
            unionfind: Default::default(),
            classes: Default::default(),
        }
    }

    pub fn slots(&self, id: Id) -> HashSet<Slot> {
        self.classes[&id].slots.clone()
    }

    pub fn normalize_enode_by_unionfind(&self, enode: &ENode) -> ENode {
        enode.map_applied_ids(|x| self.normalize_applied_id_by_unionfind(x))
    }

    // normalize i.id
    //
    // Example 1:
    // 'find(c1(s10, s11)) = c2(s11, s10)', where 'c1(s0, s1) -> c2(s1, s0)' in unionfind.
    //
    // Example 2:
    // 'find(c1(s3, s7, s8)) = c2(s8, s7)', where 'c1(s0, s1, s2) -> c2(s2, s1)' in unionfind,
    pub fn normalize_applied_id_by_unionfind(&self, i: AppliedId) -> AppliedId {
        let a = &self.unionfind[&i.id];

        // I = self.slots(i.id);
        // A = self.slots(a.id);
        // i.m   :: I -> X
        // a.m   :: A -> I
        // out.m :: A -> X
        // ==> out.m(x) = i.m(a.m(x))

        let f = |x: Slot| {
            i.m[a.m[x]]
        };

        AppliedId::new(
            a.id,
            a.m.iter().map(|(x, y)| (x, f(y))).collect(),
        )
    }

    pub fn normalize_id_by_unionfind(&self, i: Id) -> Id {
        assert!(self.classes[&i].slots.is_empty());

        self.unionfind[&i].id
    }

    pub fn ids(&self) -> Vec<Id> {
        self.unionfind.iter()
                       .filter(|(x, y)| x == &&y.id)
                       .map(|(x, _)| *x)
                       .collect()
    }

    pub fn enodes(&self, i: Id) -> HashSet<ENode> {
        let i = self.unionfind[&i].id;
        self.classes[&i].nodes.iter().map(|(x, y)| x.apply_slotmap(y)).collect()
    }

    pub fn enodes_applied(&self, i: &AppliedId) -> HashSet<ENode> {
        self.enodes(i.id).into_iter().map(|x| x.apply_slotmap(&i.m)).collect()
    }

    // TODO maybe add more?
    fn inv(&self) {
        for (i, c) in &self.classes {
            assert_eq!(self.unionfind[&i].id, *i);

            assert!(!c.slots.contains(&Slot(0)));
        }
    }
}
