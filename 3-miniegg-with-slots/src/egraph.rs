use crate::*;

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

        let old_slots = enode.slots();

        let fresh_to_old = Bijection::bijection_from_fresh_to(&old_slots);
        let fresh_enode = enode.apply_slotmap(&fresh_to_old.inverse());

        // allocate new class & slot set.
        let fresh_slots = fresh_enode.slots();
        let id = self.alloc_eclass(&fresh_slots);

        let (sh, bij) = fresh_enode.shape();
        let class_ref = self.classes.get_mut(&id).unwrap();
        class_ref.nodes.insert(sh, bij);

        AppliedId::new(id, fresh_to_old)
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
        // normalize inputs
        let l = self.normalize_applied_id_by_unionfind(l);
        let r = self.normalize_applied_id_by_unionfind(r);
        //

        // early return, if union should not be made.
        if l == r { return; }

        if l.id == r.id {
            eprintln!("We reject self-unions for now!");
            return;
        };
        //

        // make the slots fresh.
        let all_slots = l.slots().union(&r.slots()).copied().collect();
        let fresh_map = SlotMap::bijection_from_fresh_to(&all_slots).inverse();
        let l = l.apply_slotmap(&fresh_map);
        let r = r.apply_slotmap(&fresh_map);
        //

        let slots: HashSet<Slot> = l.slots().intersection(&r.slots()).copied().collect();
        let c_id = self.alloc_eclass(&slots);

        for lr in [l, r] {
            self.merge_into_eclass(lr.id, c_id, &lr.m);
        }

        // rebuild the egraph invariants
        self.fix_new_redundant_slots();
        self.fix_shape_collisions();
        //
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

    // Checks whether an EClass has an ENode not using all of its Slots. If yes, declare the missing slot "redundant".
    fn fix_new_redundant_slots(&mut self) {
        while let Some((i, sh)) = find_new_redundant_slot(self) {
            mark_redundant_slot(self, i, sh);
        }

        // the ENode (identified by its shape) only has a subset of the slots, relative to its eclass.
        fn find_new_redundant_slot(eg: &mut EGraph) -> Option<(Id, Shape)> {
            for (i, c) in &eg.classes {
                for (sh, bij) in &c.nodes {
                    if !c.slots.is_subset(&bij.values()) {
                        return Some((*i, sh.clone()));
                    }
                }
            }

            None
        }

        // This has some overlap with EGraph::union().
        fn mark_redundant_slot(eg: &mut EGraph, id: Id, sh: Shape) {
            let bij = &eg.classes[&id].nodes[&sh];

            let slots: HashSet<Slot> = eg.classes[&id].slots.intersection(&bij.values()).copied().collect();

            // TODO allocate new EClass and move everything over.
            todo!();
        }
    }

    // Checks whether two EClasses share a Shape, and if yes: unions them.
    fn fix_shape_collisions(&mut self) {
        while let Some((i1, i2, sh)) = find_shape_collision(self) {
            // X = slots(sh)
            // bij1 :: X -> slots(i1)
            // bij2 :: X -> slots(i2)
            let bij1 = &self.classes[&i1].nodes[&sh];
            let bij2 = &self.classes[&i2].nodes[&sh];

            let aid1 = AppliedId::new(i1, bij1.inverse());
            let aid2 = AppliedId::new(i2, bij2.inverse());

            self.union(aid1, aid2);
        }

        fn find_shape_collision(eg: &mut EGraph) -> Option<(Id, Id, Shape)> {
            for (i1, c1) in &eg.classes {
                for (i2, c2) in &eg.classes {
                    if i1 == i2 { continue; }

                    let l1: HashSet<&Shape> = c1.nodes.keys().collect();
                    let l2: HashSet<&Shape> = c2.nodes.keys().collect();
                    let inter: HashSet<&&Shape> = l1.intersection(&l2).collect();
                    if let Some(sh) = inter.into_iter().next() {
                        return Some((*i1, *i2, (**sh).clone()));
                    }
                }
            }

            None
        }
    }

    fn alloc_eclass(&mut self, slots: &HashSet<Slot>) -> Id {
        let c_id = Id(self.unionfind.len()); // Pick the next unused Id.
        let identity_app_id = AppliedId::new(c_id, SlotMap::identity(slots));
        let c = EClass {
            nodes: HashMap::new(),
            slots: slots.clone(),
        };
        self.classes.insert(c_id, c);
        self.unionfind.insert(c_id, identity_app_id.clone());

        c_id
    }

    // merges the EClass `from` into `to`. This deprecates the EClass `from`.
    // This does not do any cleanups!
    // map :: slots(from) -> slots(to)
    fn merge_into_eclass(&mut self, from: Id, to: Id, map: &SlotMap) {
        // X = slots(from)
        // Y = slots(to)
        // map :: X -> Y
        self.unionfind.insert(from, AppliedId::new(to, map.inverse()));
        self.fix_unionfind();

        // move enodes over.
        let from_class = self.classes.remove(&from).unwrap();
        let to_ref = self.classes.get_mut(&to).unwrap();
        for (sh, bij) in from_class.nodes {
            // SH = slots(sh)
            // bij :: SH -> X

            // out_bij :: SH -> Y
            let mut out_bij = bij.compose_partial(map);

            // map redundant slots too.
            for x in sh.slots() {
                if !out_bij.contains_key(x) {
                    out_bij.insert(x, Slot::fresh());
                }
            }

            to_ref.nodes.insert(sh, out_bij);
        }
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

            assert!(!c.slots.contains(&Slot(0)));
        }
    }
}
