use crate::*;

impl<L: Language> EGraph<L> {
    // creates a new eclass with slots "l.slots() cap r.slots()".
    // returns whether it actually did something.
    pub fn union(&mut self, l: &AppliedId, r: &AppliedId) -> bool {
        let proof = self.prove_explicit(l, r, None);
        let out = self.union_internal(l, r, proof);
        out
    }

    fn union_internal(&mut self, l: &AppliedId, r: &AppliedId, proof: ProvenEq) -> bool {
        // normalize inputs
        let l = self.find_applied_id(&l);
        let r = self.find_applied_id(&r);

        // early return, if union should not be made.
        if self.eq(&l, &r) { return false; }

        let cap = &l.slots() & &r.slots();

        if l.slots() != cap {
            self.shrink_slots(&l, &cap);
            self.union_internal(&l, &r, proof);
            return true;
        }

        if r.slots() != cap {
            self.shrink_slots(&r, &cap);
            self.union_internal(&l, &r, proof);
            return true;
        }

        if l.id == r.id {
            let id = l.id;

            // l.m :: slots(id) -> X
            // r.m :: slots(id) -> X
            // perm :: slots(id) -> slots(id)
            let perm = l.m.compose(&r.m.inverse());
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
            // sort, s.t. size(l) >= size(r).
            let size = |i| {
                let c = &self.classes[&i];
                c.nodes.len() + c.usages.len()
            };

            if size(l.id) < size(r.id) {
                self.move_to(&l, &r, proof)
            } else {
                let proof = self.prove_symmetry(proof);
                self.move_to(&r, &l, proof)
            }
            true
        }
    }

    fn shrink_slots(&mut self, from: &AppliedId, cap: &HashSet<Slot>) {
        let (id, cap) = {
            // from.m :: slots(from.id) -> X
            // cap :: set X

            // m_inv :: X -> slots(from.id)
            let m_inv = from.m.inverse();

            // cap :: set slots(from.id)
            let new_cap: HashSet<Slot> = cap.iter().map(|x| m_inv[*x]).collect();

            (from.id, new_cap)
        };

        // cap :: set slots(id)

        let c = self.classes.get_mut(&id).unwrap();
        let grp = &c.group;

        let mut final_cap = cap.clone();

        // d is a newly redundant slot.
        for d in &c.slots - &cap {
            // if d is redundant, then also the orbit of d is redundant.
            final_cap = &final_cap - &grp.orbit(d);
        }

        c.slots = cap.clone();
        let restrict = |perm: Perm| {
            let out: Perm = perm.into_iter()
                .filter(|(x, _)| cap.contains(x))
                .collect();
            out
        };
        let generators = c.group.generators().into_iter().map(restrict).collect();
        c.group = Group::new(&cap, generators);

        self.convert_eclass(from.id);
    }

    // moves everything from `from` to `to`.
    fn move_to(&mut self, from: &AppliedId, to: &AppliedId, proof: ProvenEq) {
        let map = to.m.compose_partial(&from.m.inverse());
        self.unionfind_set(from.id, proof);
        self.convert_eclass(from.id);
    }

    // Remove everything that references this e-class, and then re-add it using "semantic_add".
    // Is typically called on e-classes that point to another e-class in the unionfind.
    fn convert_eclass(&mut self, from: Id) {
        let mut adds: Vec<(L, AppliedId, AppliedId)> = Vec::new();

        // - remove all of its e-nodes
        let from_nodes = self.classes.get(&from).unwrap().nodes.clone();
        let from_id = self.mk_identity_applied_id(from);
        for (sh, (bij, src_id)) in from_nodes {
            let enode = sh.apply_slotmap(&bij);
            self.raw_remove_from_class(from, (sh, bij));
            adds.push((enode, from_id.clone(), src_id));
        }

        // - remove all of its usages
        let from_usages = self.classes.get(&from).unwrap().usages.clone();
        for sh in from_usages {
            let k = self.hashcons[&sh];
            let (bij, src_id) = self.classes[&k].nodes[&sh].clone();
            let enode = sh.apply_slotmap(&bij);
            self.raw_remove_from_class(k, (sh, bij));
            let applied_k = self.mk_identity_applied_id(k);
            adds.push((enode, applied_k, src_id));
        }

        // re-add everything.
        for (enode, j, src_id) in adds {
            self.semantic_add(&enode, &j, src_id);
        }


        // re-add the group equations as well.

        // This basically calls self.union(from, from * perm) for each perm generator in the group of from.
        let from = self.mk_identity_applied_id(from);
        let to = self.find_applied_id(&from);
        // from.m :: slots(from.id) -> C
        // to.m :: slots(to.id) -> C

        // f :: slots(from.id) -> slots(to.id)
        // Note that f is a partial map, because some slots might have become redundant.
        let f = from.m.compose_partial(&to.m.inverse());
        let change_permutation_from_from_to_to = |x: Perm| -> Perm {
            let perm: Perm = x.iter().filter_map(|(x, y)| {
                if f.contains_key(x) && f.contains_key(y) {
                    Some((f[x], f[y]))
                } else { None }
            }).collect();

            if CHECKS {
                assert!(perm.is_perm());
                assert_eq!(perm.keys(), self.classes[&to.id].slots);
            }

            perm
        };

        let set = self.classes[&from.id].group.generators()
            .into_iter()
            .map(change_permutation_from_from_to_to)
            .collect();
        self.classes.get_mut(&to.id).unwrap().group.add_set(set);
    }

    pub fn semantic_add(&mut self, enode: &L, i: &AppliedId, src_id: AppliedId) {
        let mut enode = self.find_enode(&enode);
        let mut i = self.find_applied_id(i);
        if !i.slots().is_subset(&enode.slots()) {
            let cap = &enode.slots() & &i.slots();
            self.shrink_slots(&i, &cap);

            enode = self.find_enode(&enode);
            i = self.find_applied_id(&i);
        }

        let t = self.shape(&enode);

        // upwards merging found a match!
        if let Some(j) = self.lookup_internal(&t) {
            self.union_internal(&i, &j, ProvenEqRaw::null()); // TODO this ProvenEq should be congruence!
            return;
        }

        let (sh, bij) = t;
        let mut m = i.m.inverse();

        for x in bij.values() {
            if !m.contains_key(x) {
                m.insert(x, Slot::fresh());
            }
        }
        let bij = bij.compose(&m);
        let t = (sh, bij);
        self.raw_add_to_class(i.id, t.clone(), src_id);

        self.determine_self_symmetries(i.id, t);
    }

    // finds self-symmetries in the e-class i, caused by the e-node t.
    fn determine_self_symmetries(&mut self, i: Id, t: (L, Bijection)) {
        let (sh, bij) = t;
        let enode = sh.apply_slotmap(&bij);
        for n in self.get_group_compatible_variants(&enode) {
            let (sh2, bij2) = n.weak_shape();
            if sh2 == sh {
                let grp = &mut self.classes.get_mut(&i).unwrap().group;

                // I'm looking for an equation like i == i * BIJ to add BIJ to the group.

                // - i == sh * bij == enode == n
                // - sh2 * bij2 == n
                // - sh2 == sh

                // simplify:
                // - i == sh * bij
                // - i == sh * bij2

                // -> i == i * bij^-1 * bij2
                grp.add(bij.inverse().compose(&bij2));
            }
        }
    }

}
