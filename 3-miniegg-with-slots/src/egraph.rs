use crate::*;

#[derive(Clone)]
struct EClass {
    // The set of equivalent ENodes that make up this eclass.
    nodes: HashSet<ENode>,

    // The set of slots of this eclass is [0..slotcount].
    // All larger slots are considered "redundant" (or they have to be qualified by a ENode::Lam).
    slotcount: usize,
}

// If two ENodes (that are in the EGraph) have equal ENode::shape(), they have to be in the same eclass.
pub struct EGraph {
    unionfind: HashMap<Id, AppliedId>, // normalizes the eclass. is "idempotent".
    classes: HashMap<Id, EClass>, // only ids with unionfind[x].id = x are contained.
}

impl EGraph {
    pub fn new() -> Self {
        EGraph {
            unionfind: Default::default(),
            classes: Default::default(),
        }
    }

    pub fn add_expr(&mut self, re: RecExpr) -> Id {
        todo!()
    }

    fn normalize_enode(&self, enode: ENode) -> ENode {
        enode.map_ids(|x| self.find(x))
    }

    pub fn add(&mut self, enode: ENode) -> AppliedId {
        let enode = self.normalize_enode(enode);

        if let Some(x) = self.lookup(&enode) {
            return x;
        }

        // TODO alloc new eclass for it.
        todo!()
    }

    pub fn lookup(&self, enode: &ENode) -> Option<AppliedId> {
        todo!()
    }

    pub fn find(&self, i: AppliedId) -> AppliedId {
        todo!()
    }

    pub fn union(&mut self, l: AppliedId, r: AppliedId) {
        unimplemented!()
    }
}
