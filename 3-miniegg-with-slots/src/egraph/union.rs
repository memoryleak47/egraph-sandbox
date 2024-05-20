use crate::*;

impl<L: Language> EGraph<L> {
    // creates a new eclass with slots "l.slots() cap r.slots()".
    // returns whether it actually did something.
    pub fn union(&mut self, l: &AppliedId, r: &AppliedId) -> bool {
        let out = self.union_internal(l, r);
        out
    }

    fn union_internal(&mut self, l: &AppliedId, r: &AppliedId) -> bool {
        // normalize inputs
        let l = self.find_applied_id(&l);
        let r = self.find_applied_id(&r);

        // early return, if union should not be made.
        if l == r { return false; }

        let cap = &l.slots() & &r.slots();

        // sort, s.t. size(l) >= size(r).
        let size = |i| {
            let c = &self.classes[&i];
            c.nodes.len() + c.usages.len()
        };

        let (l, r) = if size(l.id) >= size(r.id) { (l, r) } else { (r, l) };

        if l.slots() == cap {
            self.merge_into_eclass(&r, &l)
        } else if r.slots() == cap {
            self.merge_into_eclass(&l, &r)
        } else {
            let c = self.alloc_eclass_fresh(&cap);
            self.merge_into_eclass(&l, &c);
            self.merge_into_eclass(&r, &c);
            true
        }
    }

    fn merge_into_eclass(&mut self, from: &AppliedId, to: &AppliedId) -> bool {
        let from = self.find_applied_id(from);
        let to = self.find_applied_id(to);

        // self-symmetries:
        if from.id == to.id {
            let id = from.id;

            let fm = &from.m; // slots(id) -> X
            let tm = &to.m; // slots(id) -> X
            let perm = fm.compose_partial(&tm.inverse());
            assert!(perm.is_perm());
            assert_eq!(&perm.keys(), &self.classes[&id].slots);

            let grp = &mut self.classes.get_mut(&id).unwrap().group;
            if grp.contains(&perm) { return false; }

            grp.add(perm);

            true
        } else {
            let map = to.m.compose_partial(&from.m.inverse());

            self.unionfind.set(from.id, &self.mk_applied_id(to.id, map));
            self.convert_eclass(from.id);

            true
        }
    }

    // Remove everything that references this e-class, and then re-add it using "semantic_add".
    fn convert_eclass(&mut self, from: Id) {
        let mut adds: Vec<(L, AppliedId)> = Vec::new();

        // - remove all of its e-nodes
        let from_nodes = self.classes.get(&from).unwrap().nodes.clone();
        let from_id = self.mk_identity_applied_id(from);
        for (sh, bij) in from_nodes {
            let enode = sh.apply_slotmap(&bij);
            self.raw_remove_from_class(from, (sh, bij));
            adds.push((enode, from_id.clone()));
        }

        // - remove all of its usages
        let from_usages = self.classes.get(&from).unwrap().usages.clone();
        for sh in from_usages {
            let k = self.hashcons[&sh];
            let bij = self.classes[&k].nodes[&sh].clone();
            let enode = sh.apply_slotmap(&bij);
            self.raw_remove_from_class(k, (sh, bij));
            let applied_k = self.mk_identity_applied_id(k);
            adds.push((enode, applied_k));
        }


        // re-add everything.
        for (enode, j) in adds {
            self.semantic_add(&enode, &j);
        }

    }

    // self.check() should hold before and after this.
    fn semantic_add(&mut self, enode: &L, i: &AppliedId) {
        let mut enode = self.find_enode(&enode);
        let mut i = self.find_applied_id(i);

        if let Some(j) = self.lookup_internal(&enode) {
            self.union_internal(&i, &j);
        } else {
            if !i.slots().is_subset(&enode.slots()) {
                let cap = &enode.slots() & &i.slots();
                let c = self.alloc_eclass_fresh(&cap);
                self.union_internal(&c, &i);

                enode = self.find_enode(&enode);
                i = self.find_applied_id(&i);
            }

            let (sh, bij) = enode.shape();
            let mut m = i.m.inverse();

            for x in bij.values() {
                if !m.contains_key(x) {
                    m.insert(x, Slot::fresh());
                }
            }
            let bij = bij.compose(&m);
            self.raw_add_to_class(i.id, (sh, bij));
        }
    }
}
