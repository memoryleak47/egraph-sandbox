use crate::*;

// synt add:
impl<L: Language> EGraph<L> {
    pub fn add_synt_expr(&mut self, re: RecExpr<L>) -> AppliedId {
        let mut n = re.node;
        let mut refs: Vec<&mut AppliedId> = n.applied_id_occurences_mut();
        if CHECKS {
            assert_eq!(re.children.len(), refs.len());
        }
        for (i, child) in re.children.into_iter().enumerate() {
            *(refs[i]) = self.add_synt_expr(child);
        }
        self.add_synt(n)
    }

    pub fn add_synt(&mut self, enode: L) -> AppliedId {
        if let Some(x) = self.lookup_synt(&enode) {
            return x;
        }

        let t = self.shape(&enode);
        let sem_lookup = self.lookup_internal(&t);
        let i = self.mk_singleton_class(t, enode);

        if let Some(sem) = sem_lookup {
            // TODO: This shouldn't be an explicit union, but a congruence proof!
            self.union(&sem, &i);
        }

        i
    }

    fn lookup_synt(&self, enode: &L) -> Option<AppliedId> {
        let (sh, bij) = enode.weak_shape();
        let i = self.synt_hashcons.get(&sh)?;
        let i = i.apply_slotmap(&bij);
        Some(i)
    }
}

// normal add:
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
        self.add_internal(self.shape(&enode))
    }

    // self.add(x) = y implies that x.slots() is a superset of y.slots().
    // x.slots() - y.slots() are redundant slots.
    pub fn add_internal(&mut self, t: (L, SlotMap)) -> AppliedId {
        if let Some(x) = self.lookup_internal(&t) {
            return x;
        }

        let enode = t.0.apply_slotmap(&t.1);
        let enode = self.syntify_enode(enode);

        self.mk_singleton_class(t, enode)
    }

    fn syntify_app_id(&self, app: AppliedId) -> AppliedId {
        let mut app = app;
        for s in self.synt_slots(app.id) {
            if !app.m.contains_key(s) {
                app.m.insert(s, Slot::fresh());
            }
        }
        app
    }

    fn syntify_enode(&self, enode: L) -> L {
        enode.map_applied_ids(|app| self.syntify_app_id(app))
    }

    pub fn lookup(&self, n: &L) -> Option<AppliedId> {
        self.lookup_internal(&self.shape(n))
    }

    pub(in crate::egraph) fn lookup_internal(&self, (shape, n_bij): &(L, Bijection)) -> Option<AppliedId> {
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
}

impl<L: Language> EGraph<L> {
    fn mk_singleton_class(&mut self, (sh, bij): (L, SlotMap), synt_enode: L) -> AppliedId {
        let old_slots = bij.values();

        let fresh_to_old = Bijection::bijection_from_fresh_to(&old_slots);
        let old_to_fresh = fresh_to_old.inverse();

        // allocate new class & slot set.
        let fresh_slots = old_to_fresh.values();
        let i = self.alloc_eclass(&fresh_slots);

        self.raw_add_to_class(i, (sh, old_to_fresh));
        self.add_synt_enode(i, synt_enode);
        self.mk_applied_id(i, fresh_to_old)
    }

    fn add_synt_enode(&mut self, i: Id, synt_enode: L) {
        let (sh, bij) = synt_enode.weak_shape();

        if CHECKS {
            assert!(!self.synt_hashcons.contains_key(&sh));
        }

        self.classes.get_mut(&i).unwrap().synt_enode = Some(synt_enode);

        let app_id = AppliedId::new(i, bij);
        self.synt_hashcons.insert(sh, app_id);
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

impl<L: Language> EGraph<L> {
    // TODO make the public API auto "fresh" slots.
    pub fn alloc_empty_eclass(&mut self, slots: &HashSet<Slot>) -> Id {
        panic!("Can't use alloc_empty_eclass if explanations are enabled!");
        self.alloc_eclass(slots)
    }

    pub(in crate::egraph) fn alloc_eclass(&mut self, slots: &HashSet<Slot>) -> Id {
        let c_id = Id(self.unionfind.len()); // Pick the next unused Id.
        let c = EClass {
            nodes: HashMap::default(),
            group: Group::identity(&slots),
            slots: slots.clone(),
            usages: HashSet::default(),
            redundancy_proof: None,
            synt_enode: None,
        };
        self.classes.insert(c_id, c);
        self.unionfind.set(c_id, &self.mk_identity_applied_id(c_id));

        c_id
    }

    pub(in crate::egraph) fn alloc_eclass_fresh(&mut self, slots: &HashSet<Slot>) -> AppliedId {
        let bij = SlotMap::bijection_from_fresh_to(slots);
        let id = self.alloc_eclass(&bij.keys());

        self.mk_applied_id(id, bij)
    }
}
