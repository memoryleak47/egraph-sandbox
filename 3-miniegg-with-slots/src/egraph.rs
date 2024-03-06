use crate::*;

#[derive(Clone, Debug)]
struct EClass {
    // The set of equivalent ENodes that make up this eclass.
    nodes: HashSet<ENode>,

    // All other slots are considered "redundant" (or they have to be qualified by a ENode::Lam).
    slots: HashSet<Slot>,

    // The group of permutations that don't change the meaning.
    // Eg. for a commutative class, c(x, y) = c(y, x). The perm [x -> y, y -> x] would be part of this group.
    perm_group: PermGroup,
}

// invariants:
// 1. If two ENodes (that are in the EGraph) have equal ENode::shape(), they have to be in the same eclass.
// 2. enode.slots() is always a superset of c.slots, if enode is within the eclass c.
//    if ENode::Lam(si) = enode, then we require i to not be in c.slots.
// 3. AppliedId::m is always a bijection. (eg. c1(s0, s1, s0) is illegal!)
// 4. if applied_id_enodes(i) intersects applied_id_enodes(j), then applied_id_enodes(i) = applied_id_enodes(j).
//    ENode equality is hereby understood as ENode::equal_within_exposed(_, i.slots()).
//    TODO what if i.slots() != j.slots()?
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

    // returns Id instead of AppliedId, as the re isn't allowed to have free variables.
    pub fn add_expr(&mut self, re: RecExpr) -> Id {
        // re[i] should be "conceptually equivalent" to v[i].
        let mut v: Vec<AppliedId> = Vec::new();

        for enode in &re.node_dag {
            // `enode = ENode::Lam(si, a);` might be the hard part.
            let enode = enode.map_applied_ids(|a: AppliedId| {
                // a is an AppliedId to be interpreted within `re`.
                // - a.id.0 expresses the index in re.node_dag where you can find the underlying ENode `a_enode`, and
                // - a.m maps its internal slots (`a_enode.slots()`) to its exposed slots.
                let a_enode = re.node_dag[a.id.0].clone();
                assert_eq!(a.m.keys(), a_enode.slots()); // we call this set I.

                // v_a is an AppliedId to be interpreted within the EGraph.
                // It shares the same exposed slots as `a_enode`.
                let v_a: AppliedId = v[a.id.0].clone();
                // assert_eq!(v_a.slots(), a_enode.slots()); // might also be a subset relation.

                // I = a_enode.slots() = a.m.keys() = v_a.slots() = AppliedId(a.id, identity) in re;
                // EX = a.m.values() = a.slots() union out.slots()
                //      The set of slots that we want to expose in this function.
                //      Should be a subset of what `a` exposes.
                // V1 = self.slots(v_a.id) = v_a.m.keys();
                // a.m :: I -> EX;
                // v_a.m :: V1 -> I;

                // f :: V1 -> EX;
                let f = |x| a.m[v_a.m[x]];

                AppliedId::new(
                    v_a.id,
                    self.slots(v_a.id).iter().map(|x| (*x, f(*x))).collect(),
                )
            });
            v.push(self.add(enode));
        }

        let res = v.pop().unwrap();
        assert!(res.m.is_empty(), "Free variables are not allowed!");

        res.id
    }

    // TODO also has to consider the perm_group of each AppliedId.
    // or does it? I don't think this is necessary.
    fn normalize_enode(&self, enode: &ENode) -> ENode {
        enode.map_applied_ids(|x| self.find(x))
    }

    // self.add(x) = y implies that x.slots() is a superset of y.slots().
    // x.slots() - y.slots() are redundant slots.
    pub fn add(&mut self, enode: ENode) -> AppliedId {
        let enode = self.normalize_enode(&enode);

        if let Some(x) = self.lookup(&enode) {
            return x;
        }

        // allocate eclass.
        // TODO allocate new slots for everything here too (exposed & lambdas).
        let id = Id(self.classes.len());
        let slots = enode.slots();

        let app_id = AppliedId::new(id, SlotMap::identity(&slots));

        let eclass = EClass {
            nodes: HashSet::from([enode]),
            perm_group: PermGroup::identity(&slots),
            slots,
        };
        self.classes.insert(id, eclass);
        self.unionfind.insert(id, app_id.clone());

        app_id
    }

    pub fn lookup(&self, n: &ENode) -> Option<AppliedId> {
        let n = self.normalize_enode(n);

        for (i, c) in &self.classes {
            if self.unionfind[i].id != *i { continue; }

            for enode in &c.nodes {
                if enode.shape() == n.shape() {
                    let a1 = enode.slot_occurences_of_flexible();
                    let a2 = n.slot_occurences_of_flexible();
                    assert_eq!(a1.len(), a2.len());

                    let mut slotmap = SlotMap::new();
                    for (x, y) in a1.into_iter().zip(a2) {
                        if slotmap.contains_key(x) && slotmap[x] != y { panic!(); }
                        slotmap.insert(x, y);
                    }
                    let app_id = AppliedId::new(*i, slotmap);
                    return Some(app_id);
                }
            }
        }

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

    pub fn find_id(&self, i: Id) -> Id {
        assert!(self.classes[&i].slots.is_empty());

        self.unionfind[&i].id
    }

    // creates a new eclass with slots intersection(l.slots(), r.slots).
    pub fn union(&mut self, l: AppliedId, r: AppliedId) {
        let l = self.find(l);
        let r = self.find(r);

        if self.equal(&l, &r) { return; }

        let slots: HashSet<Slot> = l.slots().intersection(&r.slots()).copied().collect();

        let id = Id(self.classes.len());
        let app_id = AppliedId::new(id, SlotMap::identity(&slots));
        let eclass = EClass {
            nodes: HashSet::new(),
            perm_group: PermGroup::identity(&slots), // TODO this is wrong.
            slots,
        };
        self.classes.insert(id, eclass);
        self.unionfind.insert(id, app_id.clone());

        let mut call = |a: AppliedId, future_unions: &mut Vec<(AppliedId, AppliedId)>| {
            self.unionfind.insert(a.id, AppliedId::new(id, todo!()));
            self.fix_unionfind();

            // next steps:
            // - move the old ENodes over.
            // - upwards merging.

            todo!()
        };

        let mut future_unions = Vec::new();
        call(l, &mut future_unions);
        call(r, &mut future_unions);

        for (x, y) in future_unions {
            self.union(x, y);
        }
    }

    fn fix_unionfind(&mut self) {
        // recursively applies find() until convergence.
        let full_find = |mut x: AppliedId| {
            loop {
                let y = self.find(x.clone());
                if x == y { return x; }
                x = y;
            }
        };
        self.unionfind = self.unionfind.iter()
                        .map(|(x, y)| (*x, full_find(y.clone())))
                        .collect();
    }

    pub fn ids(&self) -> Vec<Id> {
        self.unionfind.iter()
                       .filter(|(x, y)| x == &&y.id)
                       .map(|(x, _)| *x)
                       .collect()
    }

    pub fn enodes(&self, i: Id) -> HashSet<ENode> {
        let i = self.unionfind[&i].id;
        self.classes[&i].nodes.clone()
    }

    pub fn applied_id_enodes(&self, app_id: AppliedId) -> HashSet<ENode> {
        let i = self.unionfind[&app_id.id].id;
        self.classes[&i].nodes.iter()
                              .map(|x| x.apply_slotmap(&app_id.m))
                              .collect()
    }

    // returns whether applied_id_enodes(i1) and applied_id_enodes(i2) are semantically equivalent.
    pub fn equal(&self, i1: &AppliedId, i2: &AppliedId) -> bool {
        todo!()
    }
}
