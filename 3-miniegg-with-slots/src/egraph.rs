use crate::*;

#[derive(Clone, Debug)]
pub struct EClass {
    // The set of equivalent ENodes that make up this eclass.
    // for (sh, bij) in nodes; sh.apply_slotmap(bij) represents the actual ENode.
    pub nodes: HashMap<Shape, Bijection>,

    // All other slots are considered "redundant" (or they have to be qualified by a ENode::Lam).
    pub slots: HashSet<Slot>,
}

// invariants:
// 1. If two ENodes (that are in the EGraph) have equal .shape(), they have to be in the same eclass.
// 2. enode.slots() is always a superset of c.slots, if enode is within the eclass c.
//    if ENode::Lam(si) = enode, then we require i to not be in c.slots.
// 3. AppliedId::m is always a bijection. (eg. c1(s0, s1, s0) is illegal!)
#[derive(Debug)]
pub struct EGraph {
    // an entry (l, r(sa, sb)) in unionfind corresponds to the equality l(s0, s1, s2) = r(sa, sb), where sa, sb in {s0, s1, s2}.
    // normalizes the eclass.
    // Each Id i that is an output of the unionfind itself has unionfind[i] = (i, identity()).
    pub unionfind: HashMap<Id, AppliedId>,

    // only ids with unionfind[x].id = x are contained.
    // TODO make things private by adding a recursive EGraph module that contains its implementation details.
    pub classes: HashMap<Id, EClass>,
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

    // self.add(x) = y implies that x.slots() is a superset of y.slots().
    // x.slots() - y.slots() are redundant slots.
    pub fn add(&mut self, enode: ENode) -> AppliedId {
        let enode = self.normalize_enode_by_unionfind(&enode);

        if let Some(x) = self.lookup(&enode) {
            return x;
        }

        // allocate eclass.
        // TODO allocate new slots for everything here too (exposed & lambdas).
        let id = Id(self.classes.len());
        let slots = enode.slots();

        let app_id = AppliedId::new(id, SlotMap::identity(&slots));

        let (sh, bij) = enode.shape();
        let eclass = EClass {
            nodes: HashMap::from([(sh, bij)]),
            slots,
        };
        self.classes.insert(id, eclass);
        self.unionfind.insert(id, app_id.clone());

        app_id
    }

    pub fn lookup(&self, n: &ENode) -> Option<AppliedId> {
        let n = self.normalize_enode_by_unionfind(n);
        let (shape, n_bij) = n.shape();

        for (i, c) in &self.classes {
            if let Some(cn_bij) = c.nodes.get(&shape) {
                // X = shape.slots()
                // Y = n.slots()
                // Z = c.slots()
                // n_bij :: X -> Y
                // cn_bij :: X -> Z
                // out :: Z -> Y
                let out = cn_bij.inverse().compose(&n_bij);

                let app_id = AppliedId::new(
                    *i,
                    out,
                );

                return Some(app_id);
            }
        }

        None
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

    // creates a new eclass with slots intersection(l.slots(), r.slots).
    pub fn union(&mut self, l: AppliedId, r: AppliedId) {
        let l = self.normalize_applied_id_by_unionfind(l);
        let r = self.normalize_applied_id_by_unionfind(r);

        if l == r { return; }

        let slots: HashSet<Slot> = l.slots().intersection(&r.slots()).copied().collect();

        let id = Id(self.classes.len());
        let app_id = AppliedId::new(id, SlotMap::identity(&slots));
        let eclass = EClass {
            nodes: HashMap::new(),
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
        // recursively applies normalize_applied_id_by_unionfind() until convergence.
        let full_find = |mut x: AppliedId| {
            loop {
                let y = self.normalize_applied_id_by_unionfind(x.clone());
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
        self.classes[&i].nodes.iter().map(|(x, y)| x.apply_slotmap(y)).collect()
    }

    // TODO maybe add more?
    fn inv(&self) {
        for (i, c) in &self.classes {
            assert_eq!(self.unionfind[&i].id, *i);
        }
    }
}
