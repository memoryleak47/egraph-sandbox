use crate::*;

impl<L: Language> EGraph<L> {
    // creates a new eclass with slots "l.slots() cap r.slots()".
    // returns whether it actually did something.
    pub fn union(&mut self, l: &AppliedId, r: &AppliedId) -> bool {
        let proof = prove_explicit(l, r, None);
        let out = self.union_internal(l, r, proof);

        out
    }

    pub(in crate::egraph) fn union_internal(&mut self, l: &AppliedId, r: &AppliedId, proof: ProvenEq) -> bool {
        // normalize inputs
        let (l, p_l) = self.proven_find_applied_id(&l);
        let (r, p_r) = self.proven_find_applied_id(&r);

        let a = prove_symmetry(p_l);
        let a = prove_transitivity(a, proof);
        let a = prove_transitivity(a, p_r);
        let proof = a;
        if CHECKS {
            assert_eq!(proof.l.id, l.id);
            assert_eq!(proof.r.id, r.id);
        }

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

            let proven_perm = ProvenPerm(perm, proof);
            assert_eq!(proven_perm.1.l.id, id);

            proven_perm.check(self);
            let grp = &mut self.classes.get_mut(&id).unwrap().group;
            if grp.contains(&proven_perm.to_slotmap()) { return false; }

            grp.add(proven_perm);

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
                let proof = prove_symmetry(proof);
                self.move_to(&r, &l, proof)
            }
            true
        }
    }

    fn shrink_slots(&mut self, from: &AppliedId, cap: &HashSet<Slot>) {
        panic!("For now, we don't support shrinking");
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
        let restrict_proven = |proven_perm: ProvenPerm| {
            proven_perm.check(self);
            let out = ProvenPerm(restrict(proven_perm.0), proven_perm.1);
            out.check(self);
            out
        };
        let generators = c.group.generators().into_iter().map(restrict_proven).collect();
        let syn_slots = &self.syn_slots(id);
        let identity = ProvenPerm::identity(id, &cap, syn_slots);
        identity.check(self);
        c.group = Group::new(&identity, generators);

        self.convert_eclass(from.id);
    }

    // moves everything from `from` to `to`.
    fn move_to(&mut self, from: &AppliedId, to: &AppliedId, proof: ProvenEq) {
        if CHECKS {
            assert_eq!(from.id, proof.l.id);
            assert_eq!(to.id, proof.r.id);
        }
        let map = to.m.compose_partial(&from.m.inverse());
        let app_id = self.mk_applied_id(to.id, map);
        self.unionfind_set(from.id, app_id, proof);
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
        let (_, prf) = self.proven_find_applied_id(&from);
        let prf_rev = prove_symmetry(prf.clone());
        let change_proven_permutation_from_from_to_to = |proven_perm: ProvenPerm| {
            let new_perm = change_permutation_from_from_to_to(proven_perm.0);
            let new_proof = prove_transitivity(prf_rev.clone(), prove_transitivity(proven_perm.1, prf.clone()));
            ProvenPerm(new_perm, new_proof)
        };

        let set = self.classes[&from.id].group.generators()
            .into_iter()
            .map(change_proven_permutation_from_from_to_to)
            .collect();
        self.classes.get_mut(&to.id).unwrap().group.add_set(set);
    }

    pub fn semantic_add(&mut self, enode: &L, i_orig: &AppliedId, src_id: AppliedId) {
        let mut enode = self.find_enode(&enode);
        let mut i = self.find_applied_id(i_orig);
        // i.m :: slots(i) -> X
        // i_orig.m :: slots(i_orig) -> X
        // old src_id.m :: slots(src_id.id) -> slots(i_orig)
        // new src_id.m :: slots(src_id.id) -> slots(i)
        let theta = i_orig.m.compose(&i.m.inverse());
        let src_id = src_id.apply_slotmap_fresh(&theta);
        if !i.slots().is_subset(&enode.slots()) {
            let cap = &enode.slots() & &i.slots();
            self.shrink_slots(&i, &cap);

            enode = self.find_enode(&enode);
            i = self.find_applied_id(&i);
        }

        let t = self.shape(&enode);

        // upwards merging found a match!
        if self.lookup_internal(&t).is_some() {
            self.handle_congruence(&src_id);
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
        self.raw_add_to_class(i.id, t.clone(), src_id.clone());

        self.determine_self_symmetries(src_id.id);
    }

    // finds self-symmetries caused by the e-node `src_id`.
    fn determine_self_symmetries(&mut self, src_id: Id) {
        let (leader, leader_prf) = self.proven_unionfind_get(src_id);
        let neg_leader_prf = prove_symmetry(leader_prf.clone());
        let i = leader.id;
        let leader_bij = leader.m;

        let syn_slots = self.syn_slots(src_id);
        let src_identity = AppliedId::new(src_id, Perm::identity(&syn_slots));
        let syn_node = self.get_syn_node(&src_identity);
        if CHECKS {
            assert_eq!(&syn_slots, &syn_node.slots());
        }

        let (enode, prfs) = self.proven_find_enode(&syn_node);
        let (weak, bij) = enode.weak_shape();
        for (n, prfs2) in self.proven_get_group_compatible_variants(&enode) {
            let (weak2, bij2) = n.weak_shape();
            if weak == weak2 {
                // I'm looking for an equation like i == i * BIJ to add BIJ to the group.

                // - i == sh * bij == enode == n
                // - sh2 * bij2 == n
                // - sh2 == sh

                // simplify:
                // - i == sh * bij
                // - i == sh * bij2

                // -> i == i * bij^-1 * bij2

                let perm = bij.inverse().compose(&bij2);
                if CHECKS { assert!(perm.is_perm()); }

                let l = src_identity.clone();
                let r = l.apply_slotmap(&perm);
                let eq = Equation { l, r };

                let mut combined_prfs = Vec::new();
                for (old_to_new_ids, perm_prf) in prfs.iter().zip(prfs2.iter()) {
                    let new_to_old_ids = prove_symmetry(old_to_new_ids.clone());
                    let combined = prove_transitivity(prove_transitivity(old_to_new_ids.clone(), perm_prf.clone()), new_to_old_ids);
                    combined_prfs.push(combined);
                }

                // src_id[...] == src_id[...]
                let prf = CongruenceProof(combined_prfs).check(&eq, self).unwrap();
                assert_eq!(prf.l.id, src_id);
                assert_eq!(prf.r.id, src_id);

                // i[...] == i[...]
                let prf = prove_transitivity(neg_leader_prf.clone(), prove_transitivity(prf, leader_prf.clone()));
                let perm = leader_bij.compose(&perm.compose(&leader_bij.inverse()));

                let slots = self.slots(i);
                assert_eq!(prf.l.id, i);
                assert_eq!(prf.r.id, i);
                assert_eq!(&prf.l.m.values(), &slots);
                assert_eq!(&prf.l.m.keys(), &slots);
                assert_eq!(&prf.r.m.values(), &slots);
                assert_eq!(&prf.r.m.keys(), &slots);
                let proven_perm = ProvenPerm(perm, prf);

                assert!(proven_perm.0.is_perm());
                assert_eq!(proven_perm.0.keys(), slots);

                if CHECKS {
                    proven_perm.check(self);
                }
                let grp = &mut self.classes.get_mut(&i).unwrap().group;
                grp.add(proven_perm);
            }
        }
    }

    pub(in crate::egraph) fn handle_congruence(&mut self, a: &AppliedId) {
        let a_node = self.get_syn_node(a);
        let (t, vec_p1) = self.proven_shape(&a_node);

        let b = self.lookup_internal(&t).expect("handle_congruence should only be called on hashcons collision!");
        let (bij, /*src id*/ c) = self.classes[&b.id].nodes[&t.0].clone();
        let c = c.apply_slotmap_fresh(&b.m);
        let c_node = self.get_syn_node(&c);
        let (_, vec_p2) = self.proven_shape(&c_node);

        let mut vec = Vec::new();
        for (l, r) in vec_p1.into_iter().zip(vec_p2.into_iter()) {
            let r_inv = prove_symmetry(r);
            let l_to_r = prove_transitivity(l, r_inv);
            vec.push(l_to_r);
        }

        let proven_eq = prove_congruence(&a, &c, vec, self);
        self.union_internal(&a, &c, proven_eq);
    }
}
