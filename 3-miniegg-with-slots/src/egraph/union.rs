use crate::*;

impl<L: Language> EGraph<L> {
    // creates a new eclass with slots "l.slots() cap r.slots()".
    // returns whether it actually did something.
    pub fn union(&mut self, l: &AppliedId, r: &AppliedId) -> bool {
        let syn_l = self.synify_app_id(l.clone());
        let syn_r = self.synify_app_id(r.clone());

        let proof = self.prove_explicit(&syn_l, &syn_r, None);
        let out = self.union_internal(l, r, proof);

        out
    }

    pub(in crate::egraph) fn union_internal(&mut self, l: &AppliedId, r: &AppliedId, proof: ProvenEq) -> bool {
        // normalize inputs
        let (l, p_l) = self.proven_find_applied_id(&l);
        let (r, p_r) = self.proven_find_applied_id(&r);
        p_r.check(self);

        let a = self.prove_symmetry(p_l);
        let a = self.prove_transitivity(a, proof);
        let a = self.prove_transitivity(a, p_r);
        let proof = a;
        if CHECKS {
            assert_eq!(proof.l.id, l.id);
            assert_eq!(proof.r.id, r.id);
        }

        // early return, if union should not be made.
        if self.eq(&l, &r) { return false; }

        let cap = &l.slots() & &r.slots();

        if l.slots() != cap {
            self.shrink_slots(&l, &cap, proof.clone());
            self.union_internal(&l, &r, proof);
            return true;
        }

        if r.slots() != cap {
            let flipped_proof = self.prove_symmetry(proof.clone());
            self.shrink_slots(&r, &cap, flipped_proof);
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

            proven_perm.check();
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
                let proof = self.prove_symmetry(proof);
                self.move_to(&r, &l, proof)
            }
            true
        }
    }

    // proof.l should be i.
    // proof.r should be missing a few slots.
    fn record_redundancy_witness(&mut self, i: Id, proof: ProvenEq) {
        if CHECKS {
            assert!(self.is_alive(i));
            assert_eq!(proof.l.id, i);
        }

        let flipped = prove_symmetry(proof.clone());
        let new_prf = prove_transitivity(proof, flipped);

        let old_prf = &mut self.classes.get_mut(&i).unwrap().redundancy_proof;
        *old_prf = prove_transitivity(new_prf, old_prf.clone());
    }

    // We expect `from` to be on the lhs of this equation.
    fn shrink_slots(&mut self, from: &AppliedId, cap: &HashSet<Slot>, proof: ProvenEq) {
        if CHECKS {
            assert_eq!(from.id, proof.l.id);
        }

        self.record_redundancy_witness(from.id, proof);

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

        let syn_slots = &self.syn_slots(id);
        let c = self.classes.get_mut(&id).unwrap();
        let grp = &c.group;

        let mut final_cap = cap.clone();

        // d is a newly redundant slot.
        for d in &c.slots - &cap {
            // if d is redundant, then also the orbit of d is redundant.
            final_cap = &final_cap - &grp.orbit(d);
        }

        c.slots = cap.clone();
        let generators = c.group.generators();
        let _ = c;

        let restrict_proven = |proven_perm: ProvenPerm| {
            proven_perm.check();

            let perm = proven_perm.0.into_iter()
                .filter(|(x, _)| cap.contains(x))
                .collect();
            let prf = self.disassociate_proven_eq(proven_perm.1);
            let out = ProvenPerm(perm, prf);
            out.check();
            out
        };

        let generators = generators.into_iter().map(restrict_proven).collect();
        let identity = ProvenPerm::identity(id, &cap, syn_slots);
        identity.check();
        let c = self.classes.get_mut(&id).unwrap();
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
        let app_id = self.mk_sem_applied_id(to.id, map);
        self.unionfind_set(from.id, app_id, proof);
        self.convert_eclass(from.id);
    }

    // Remove everything that references this e-class, and then re-add it using "semantic_add".
    // Is typically called on e-classes that point to another e-class in the unionfind.
    fn convert_eclass(&mut self, from: Id) {
        let mut adds: Vec<(L, AppliedId, AppliedId)> = Vec::new();

        // - remove all of its e-nodes
        let from_nodes = self.classes.get(&from).unwrap().nodes.clone();
        let from_id = self.mk_sem_identity_applied_id(from);
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
            let applied_k = self.mk_sem_identity_applied_id(k);
            adds.push((enode, applied_k, src_id));
        }

        // re-add everything.
        for (enode, j, src_id) in adds {
            self.semantic_add(&enode, &j, src_id);
        }


        // re-add the group equations as well.

        // This basically calls self.union(from, from * perm) for each perm generator in the group of from.
        let from = self.mk_sem_identity_applied_id(from);
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
        let prf_rev = self.prove_symmetry(prf.clone());
        let change_proven_permutation_from_from_to_to = |proven_perm: ProvenPerm| {
            let new_perm = change_permutation_from_from_to_to(proven_perm.0);
            let new_proof = self.prove_transitivity(prf_rev.clone(), self.prove_transitivity(proven_perm.1, prf.clone()));
            ProvenPerm(new_perm, new_proof)
        };

        let set = self.classes[&from.id].group.generators()
            .into_iter()
            .map(change_proven_permutation_from_from_to_to)
            .collect();
        self.classes.get_mut(&to.id).unwrap().group.add_set(set);
    }

    fn handle_shrink_in_upwards_merge(&mut self, src_id: Id) {
        let (leader, leader_prf) = self.proven_unionfind_get(src_id);
        let neg_leader_prf = prove_symmetry(leader_prf.clone());
        let src_syn_slots = self.syn_slots(src_id);

        let identity = self.mk_syn_identity_applied_id(src_id);
        let syn_enode = self.get_syn_node(&identity);
        assert!(syn_enode.slots().is_subset(&src_syn_slots));
        let (new_node, prfs) = self.proven_find_enode(&syn_enode);
        assert!(new_node.slots().is_subset(&src_syn_slots));

        // the set of slots that still remain non-redundant.
        let mut fixpoint_set = HashSet::default();

        let mut combined = Vec::new();
        for (app_id, prf) in new_node.applied_id_occurences().into_iter().zip(prfs.into_iter()) {
            let rev = prove_symmetry(prf.clone());
            let cycle = prove_transitivity(prf, rev);

            for (x, y) in cycle.l.m.iter() {
                if cycle.r.m.get(x) == Some(y) {
                    let fixed = app_id.m[x];
                    if src_syn_slots.contains(&fixed) {
                        fixpoint_set.insert(fixed);
                    }
                }
            }

            combined.push(cycle);
        }
        let cong = self.prove_congruence(src_id, src_id, &combined);
        let prf = self.prove_transitivity(neg_leader_prf.clone(), self.prove_transitivity(cong, leader_prf.clone()));

        let leader_inv = leader.m.inverse();
        if CHECKS {
            let ty = self.syn_slots(src_id);
            assert!(fixpoint_set.is_subset(&ty));
            assert!(leader_inv.keys().is_subset(&ty));
        }
        let leader_fixpoint_set: HashSet<Slot> = fixpoint_set.iter().map(|x| leader_inv[*x]).collect();
        let leader_fixpoint_set = &leader_fixpoint_set & &self.slots(leader.id);
        self.shrink_slots(&leader, &leader_fixpoint_set, prf);
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
            self.handle_shrink_in_upwards_merge(src_id.id);

            enode = self.find_enode(&enode);
            i = self.find_applied_id(&i);
        }

        let t = self.shape(&enode);

        // upwards merging found a match!
        if self.lookup_internal(&t).is_some() {
            self.handle_congruence(src_id.id);
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
        let neg_leader_prf = self.prove_symmetry(leader_prf.clone());
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

                // no need to add the identity permutation.
                if perm.iter().all(|(x, y)| x == y) { continue; }

                if CHECKS { assert!(perm.is_perm()); }

                let mut combined_prfs = Vec::new();
                for (old_to_new_ids, perm_prf) in prfs.iter().zip(prfs2.iter()) {
                    let new_to_old_ids = self.prove_symmetry(old_to_new_ids.clone());

                    let eq = self.prove_transitivity(self.prove_transitivity(old_to_new_ids.clone(), perm_prf.clone()), new_to_old_ids.clone());
                    let combined = TransitivityProof(self.prove_transitivity(old_to_new_ids.clone(), perm_prf.clone()), new_to_old_ids.clone()).check(&eq);
                    combined_prfs.push(combined);
                }

                // src_id[...] == src_id[...]

                let prf = self.prove_congruence(src_id, src_id, &combined_prfs);
                if CHECKS {
                    assert_eq!(prf.l.id, src_id);
                    assert_eq!(prf.r.id, src_id);
                }

                // i[...] == i[...]
                let prf = self.prove_transitivity(neg_leader_prf.clone(), self.prove_transitivity(prf, leader_prf.clone()));
                let perm = leader_bij.compose_partial(&perm.compose_partial(&leader_bij.inverse()));

                let slots = self.slots(i);
                let syn_slots = self.syn_slots(i);
                if CHECKS {
                    assert_eq!(prf.l.id, i);
                    assert_eq!(prf.r.id, i);
                }
                let proven_perm = ProvenPerm(perm, prf);

                if CHECKS {
                    proven_perm.check();
                }
                let grp = &mut self.classes.get_mut(&i).unwrap().group;
                grp.add(proven_perm);
            }
        }
    }

    pub(in crate::egraph) fn handle_congruence(&mut self, a: Id) {
        let a = &self.mk_syn_identity_applied_id(a);
        let a_node = self.get_syn_node(a);
        let (t, vec_p1) = self.proven_shape(&a_node);

        let b = self.lookup_internal(&t).expect("handle_congruence should only be called on hashcons collision!");
        let (bij, /*src id*/ c) = self.classes[&b.id].nodes[&t.0].clone();
        let c = c.apply_slotmap_fresh(&b.m); // TODO why on earth does it fail if I remove this? This should do literally nothing.
        let c_node = self.get_syn_node(&c);
        let (t2, vec_p2) = self.proven_shape(&c_node);
        if CHECKS {
            assert_eq!(&t.0, &t2.0);
        }

        let mut vec = Vec::new();
        for (l, r) in vec_p1.into_iter().zip(vec_p2.into_iter()) {
            let r_inv = self.prove_symmetry(r);
            let l_to_r = self.prove_transitivity(l, r_inv);
            vec.push(l_to_r);
        }

        let proven_eq = self.prove_congruence(a.id, c.id, &vec);
        let eq = proven_eq.equ();
        let l = eq.l;
        let r = eq.r;
        self.union_internal(&l, &r, proven_eq);
    }
}
