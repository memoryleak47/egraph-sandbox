use crate::*;

#[derive(Clone, Debug)]
struct EClass {
    // The set of equivalent ENodes that make up this eclass.
    nodes: HashSet<ENode>,

    // All other slots are considered "redundant" (or they have to be qualified by a ENode::Lam).
    slots: HashSet<Slot>,
}

// invariants:
// 1. If two ENodes (that are in the EGraph) have equal ENode::shape(), they have to be in the same eclass.
// 2. enode.slots() is always a superset of c.slots, if enode is within the eclass c.
//    if ENode::Lam(si) = enode, then we require i to not be in c.slots.
// 3. AppliedId::m is always a bijection. (eg. c1(s0, s1, s0) is illegal!)
#[derive(Debug)]
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

    pub fn slots(&self, id: Id) -> HashSet<Slot> {
        self.classes[&id].slots.clone()
    }

    pub fn add_expr(&mut self, re: RecExpr) -> AppliedId {
        let mut v: Vec<AppliedId> = Vec::new();
        for x in re.node_dag {
            let x = x.map_ids(|a: AppliedId| {
                let v_a: AppliedId = v[a.id.0].clone();

                // A1 = re.node_dag[a.id.0].slots() = a.m.keys();
                // A2 = a.slots();
                // V1 = self.slots(v_a.id);
                // V2 = v_a.slots(); [subset of A2]
                // a.m :: A1 -> A2;
                // v_a.m :: V1 -> V2;

                // f :: V1 -> V2;
                // TODO is it really correct that this doesn't depend on a.m?
                let f = |x| v_a.m[x];

                AppliedId::new(
                    v_a.id,
                    self.slots(v_a.id).iter().map(|x| (*x, f(*x))).collect(),
                )
            });
            v.push(self.add(x));
        }

        v.pop().unwrap()
    }

    fn normalize_enode(&self, enode: &ENode) -> ENode {
        enode.map_ids(|x| self.find(x))
    }

    // self.add(x) = y implies that x.slots() is a superset of y.slots().
    // x.slots() - y.slots() are redundant slots.
    pub fn add(&mut self, enode: ENode) -> AppliedId {
        let enode = self.normalize_enode(&enode);

        if let Some(x) = self.lookup(&enode) {
            return x;
        }

        // allocate eclass.
        let id = Id(self.classes.len());
        let enode_slots = enode.slots();
        // m :: FRESH -> enode.slots()
        let m = SlotMap::bijection_from_fresh_to(&enode_slots);
        let slots = m.keys();
        let enode = enode.apply_slotmap(&m.inverse());

        let app_id = AppliedId::new(id, m);
        let app_id2 = AppliedId::new(id, SlotMap::identity(&slots));

        let eclass = EClass {
            nodes: HashSet::from([enode]),
            slots: slots,
        };
        self.classes.insert(id, eclass);
        self.unionfind.insert(id, app_id2);

        app_id
    }

    // TODO implement.
    // This implementation of `lookup` will not enforce sharing at all.
    pub fn lookup(&self, n: &ENode) -> Option<AppliedId> {
        None
    }

    // normalize i.id
    //
    // Example 1:
    // 'find(c1(s10, s11)) = c2(s11, s10)', where 'c1(s0, s1) -> c2(s1, s0)' in unionfind.
    //
    // Example 2:
    // 'find(c1(s3, s7, s8)) = c2(s8, s7)', where 'c1(s0, s1, s2) -> c2(s2, s1)' in unionfind,
    pub fn find(&self, i: AppliedId) -> AppliedId {
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

    pub fn union(&mut self, l: AppliedId, r: AppliedId) {
        unimplemented!()
    }
}
