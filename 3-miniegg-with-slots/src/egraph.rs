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
    // an entry (l, r(sa, sb)) in unionfind corresponds to the equality l(s0, s1, s2) = r(sa, sb), where sa, sb in {s0, s1, s2}.
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

    fn normalize_enode(&self, enode: &ENode) -> ENode {
        enode.map_ids(&|x| self.find(x))
    }

    pub fn add(&mut self, enode: ENode) -> AppliedId {
        let enode = self.normalize_enode(&enode);

        if let Some(x) = self.lookup(&enode) {
            return x;
        }

        // TODO alloc new eclass for it.
        todo!()
    }

    pub fn lookup(&self, enode: &ENode) -> Option<AppliedId> {
        // 1. normalize eclasses.
        let enode = self.normalize_enode(enode);

        // 2. normalize slots.
        let shape = enode.shape();

        for (&i, c) in &self.classes {
            for n in &c.nodes {
                if n.shape() == shape {
                    todo!();
                }
            }
        }

        None
    }

    // normalize i.id
    //
    // Example:
    // find(c1(s3, s7, s8)), where 'c1(s0, s1, s2) -> c2(s2, s1)' in unionfind,
    // yields c2(s8, s7).
    pub fn find(&self, i: AppliedId) -> AppliedId {
        let a: &AppliedId = &self.unionfind[&i.id];

        let f = |x: Slot| i.args[x.0];

        AppliedId::new(
            a.id,
            a.args.iter().copied().map(f).collect(),
        )
    }

    pub fn union(&mut self, l: AppliedId, r: AppliedId) {
        unimplemented!()
    }
}
