use crate::*;

impl<L: Language> EGraph<L> {
    // creates a new eclass with slots "l.slots() cap r.slots()".
    // returns whether it actually did something.
    // SIDE-EFFECT: Might add arbitrary new unions (by hashcons collisions).
    pub fn union(&mut self, l: &AppliedId, r: &AppliedId) -> bool {
        let out = self.union_internal(l, r);
        out
    }

    // SIDE-EFFECT: Might add arbitrary new unions (by hashcons collisions).
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

            // merge_into_eclass(r, c) isn't enough, as r might already be empty if l.id == r.id.
            self.union_internal(&r, &c);

            true
        }
    }

    // A directed union from `from` to `to`.
    // `from.id` gets deprecated, if it's different from `to.id`.
    //
    // Only gets called with from.slots() superset to.slots().
    // SIDE-EFFECT: Might add arbitrary new unions (by hashcons collisions).
    fn merge_into_eclass(&mut self, from: &AppliedId, to: &AppliedId) -> bool {
        if CHECKS {
            assert!(from.slots().is_superset(&to.slots()));
        }

        let from = self.find_applied_id(from);
        let to = self.find_applied_id(to);

        // move over the group perms from `from` to `to`.
        let group_grew = {
            // from.m :: slots(from.id) -> C
            // to.m :: slots(to.id) -> C
            let tmp = from.m.compose_partial(&to.m.inverse());
            let change_permutation_from_from_to_to = |x: Perm| -> Perm {
                x.iter().map(|(x, y)| (tmp[x], tmp[y])).collect()
            };

            let old_size = self.classes[&to.id].group.count();
            let set = self.classes[&from.id].group.generators()
                .into_iter()
                .map(change_permutation_from_from_to_to)
                .collect();
            self.classes.get_mut(&to.id).unwrap().group.add_set(set);
            let new_size = self.classes[&to.id].group.count();

            new_size > old_size
        };

        // self-symmetries:
        if from.id == to.id {
            let id = from.id;

            let fm = &from.m; // slots(id) -> X
            let tm = &to.m; // slots(id) -> X
            let perm = fm.compose_partial(&tm.inverse());
            if CHECKS {
                assert!(perm.is_perm());
                assert_eq!(&perm.keys(), &self.classes[&id].slots);
            }

            let grp = &mut self.classes.get_mut(&id).unwrap().group;
            if grp.contains(&perm) { return false; }

            grp.add(perm);

            self.convert_eclass(id);

            true
        } else {
            let map = to.m.compose_partial(&from.m.inverse());

            self.unionfind.set(from.id, &self.mk_applied_id(to.id, map));

            if group_grew {
                self.convert_eclass(to.id);
            }
            self.convert_eclass(from.id);

            true
        }
    }

    // Remove everything that references this e-class, and then re-add it using "semantic_add".
    // SIDE-EFFECT: Might add arbitrary new unions (by hashcons collisions).
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

    // for all AppliedIds that are contained in `enode`, permute their arguments as their groups allow.
    fn get_group_compatible_variants(&self, enode: &L) -> HashSet<L> {
        let mut s = HashSet::default();
        s.insert(enode.clone());

        for (i, app_id) in enode.applied_id_occurences().iter().enumerate() {
            let grp_perms = self.classes[&app_id.id].group.all_perms();
            let mut next = HashSet::default();
            for x in s {
                for y in &grp_perms {
                    let mut x = x.clone();
                    let rf: &mut SlotMap = &mut x.applied_id_occurences_mut()[i].m;
                    *rf = y.compose(rf);
                    next.insert(x);
                }
            }
            s = next;
        }
        s
    }

    // SIDE-EFFECT: Might add arbitrary new unions (by hashcons collisions).
    pub fn semantic_add(&mut self, enode: &L, i: &AppliedId) {
        let enode = self.find_enode(&enode);
        let i = self.find_applied_id(i);

        for enode2 in self.get_group_compatible_variants(&enode) {
            self.semantic_add_impl(&enode2, &i);
        }
    }

    // self.check() should hold before and after this.
    // SIDE-EFFECT: Might add arbitrary new unions (by hashcons collisions).
    fn semantic_add_impl(&mut self, enode: &L, i: &AppliedId) {
        let mut enode = enode.clone();
        let mut i = i.clone();

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
