use crate::*;

impl<L: Language> EGraph<L> {
    pub fn add_expr(&mut self, re: RecExpr<L>) -> AppliedId {
        let mut n = re.node;
        let mut refs: Vec<&mut AppliedId> = n.applied_id_occurences_mut();
        if CHECKS {
            assert_eq!(re.children.len(), refs.len());
        }
        for (i, child) in re.children.into_iter().enumerate() {
            *(refs[i]) = self.add_expr(child);
        }
        self.add(n)
    }

    pub fn add(&mut self, enode: L) -> AppliedId {
        self.add_internal(enode)
    }

    // self.add(x) = y implies that x.slots() is a superset of y.slots().
    // x.slots() - y.slots() are redundant slots.
    pub(in crate::egraph) fn add_internal(&mut self, enode: L) -> AppliedId {
        let enode = self.find_enode(&enode);

        if let Some(x) = self.lookup(&enode) {
            return x;
        }

        let old_slots = enode.slots();

        let fresh_to_old = Bijection::bijection_from_fresh_to(&old_slots);
        let fresh_enode = enode.apply_slotmap(&fresh_to_old.inverse());

        // allocate new class & slot set.
        let fresh_slots = fresh_enode.slots();
        let id = self.alloc_eclass(&fresh_slots);
        let app_id = self.mk_identity_applied_id(id);

        // calling semantic_add is a bit overkill here, we use it for the symmetries though.
        self.semantic_add(&fresh_enode, &app_id);

        self.mk_applied_id(id, fresh_to_old)
    }

    pub fn lookup(&self, n: &L) -> Option<AppliedId> {
        self.lookup_internal(n)
    }

    pub(in crate::egraph) fn lookup_internal(&self, n: &L) -> Option<AppliedId> {
        let n = self.find_enode(n);
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
        let out = out.iter().filter(|(x, _)| c.slots.contains(x)).collect();

        let app_id = self.mk_applied_id(
            *i,
            out,
        );

        if CHECKS {
            assert_eq!(&c.slots, &app_id.m.keys());
        }

        Some(app_id)
    }

    // TODO make this private in favor of alloc_eclass_fresh.
    pub fn alloc_eclass(&mut self, slots: &HashSet<Slot>) -> Id {
        let c_id = Id(self.unionfind.len()); // Pick the next unused Id.
        let c = EClass {
            nodes: HashMap::default(),
            group: Group::identity(&slots),
            slots: slots.clone(),
            usages: HashSet::default(),
        };
        self.classes.insert(c_id, c);
        self.unionfind.set(c_id, &self.mk_identity_applied_id(c_id));

        c_id
    }

    pub fn alloc_eclass_fresh(&mut self, slots: &HashSet<Slot>) -> AppliedId {
        let bij = SlotMap::bijection_from_fresh_to(slots);
        let id = self.alloc_eclass(&bij.keys());

        self.mk_applied_id(id, bij)
    }

    // adds (sh, bij) to the eclass `id`.
    pub(in crate::egraph) fn raw_add_to_class(&mut self, id: Id, (sh, bij): (L, Bijection)) {
        let tmp1 = self.classes.get_mut(&id).unwrap().nodes.insert(sh.clone(), bij);
        let tmp2 = self.hashcons.insert(sh.clone(), id);
        if CHECKS {
            assert!(tmp1.is_none());
            assert!(tmp2.is_none());
        }
        for ref_id in sh.ids() {
            let usages = &mut self.classes.get_mut(&ref_id).unwrap().usages;
            usages.insert(sh.clone());
        }
    }

    // TODO: the bijection here is probably useless and error-prone. Let's only get Id and Shape as arguments here.
    pub(in crate::egraph) fn raw_remove_from_class(&mut self, id: Id, (sh, _bij): (L, Bijection)) {
        let tmp1 = self.classes.get_mut(&id).unwrap().nodes.remove(&sh);
        let tmp2 = self.hashcons.remove(&sh);
        if CHECKS {
            assert!(tmp1.is_some());
            assert!(tmp2.is_some());
        }
        for ref_id in sh.ids() {
            let usages = &mut self.classes.get_mut(&ref_id).unwrap().usages;
            usages.remove(&sh);
        }
    }

}
