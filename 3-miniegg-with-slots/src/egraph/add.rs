use crate::*;

impl EGraph {
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
        self.raw_add_to_class(id, (sh.clone(), bij));

        AppliedId::new(id, fresh_to_old)
    }

    pub fn lookup(&self, n: &ENode) -> Option<AppliedId> {
        let n = self.normalize_enode_by_unionfind(n);
        let (shape, n_bij) = n.shape();
        let i = self.hashcons.get(&shape)?;
        let c = &self.classes[i];
        let cn_bij = c.nodes.get(&shape).unwrap();

        // X = shape.slots()
        // Y = n.slots()
        // Z = c.slots()
        // n_bij :: X -> Y
        // cn_bij :: X -> Z
        // out :: Z -> Y
        let out = cn_bij.inverse().compose(&n_bij);

        // Note that ENodes in an EClass can have redundant slots.
        // They shouldn't come up in the AppliedId.
        let out = out.iter().filter(|(x, y)| c.slots.contains(x)).collect();

        let app_id = AppliedId::new(
            *i,
            out,
        );

        assert_eq!(&c.slots, &app_id.m.keys());

        Some(app_id)
    }

    // TODO make this private in favor of alloc_eclass_fresh.
    pub fn alloc_eclass(&mut self, slots: &HashSet<Slot>) -> Id {
        let c_id = Id(self.unionfind.len()); // Pick the next unused Id.
        let identity_app_id = AppliedId::new(c_id, SlotMap::identity(slots));
        let c = EClass {
            nodes: HashMap::default(),
            slots: slots.clone(),
            usages: HashSet::default(),
        };
        self.classes.insert(c_id, c);
        self.unionfind.insert(c_id, identity_app_id.clone());

        c_id
    }

    pub fn alloc_eclass_fresh(&mut self, slots: &HashSet<Slot>) -> AppliedId {
        let bij = SlotMap::bijection_from_fresh_to(slots);
        let id = self.alloc_eclass(&bij.keys());

        AppliedId::new(id, bij)
    }

    // adds (sh, bij) to the eclass `id`.
    pub(in crate::egraph) fn raw_add_to_class(&mut self, id: Id, (sh, bij): (Shape, Bijection)) {
        assert!(self.classes.get_mut(&id).unwrap().nodes.insert(sh.clone(), bij).is_none());
        assert!(self.hashcons.insert(sh.clone(), id).is_none());
        for ref_id in sh.ids() {
            let usages = &mut self.classes.get_mut(&ref_id).unwrap().usages;
            usages.insert(sh.clone());
        }
    }

    pub(in crate::egraph) fn raw_remove_from_class(&mut self, id: Id, (sh, bij): (Shape, Bijection)) {
        assert!(self.classes.get_mut(&id).unwrap().nodes.remove(&sh).is_some());
        assert!(self.hashcons.remove(&sh).is_some());
        for ref_id in sh.ids() {
            let usages = &mut self.classes.get_mut(&ref_id).unwrap().usages;
            usages.remove(&sh);
        }
    }

}
