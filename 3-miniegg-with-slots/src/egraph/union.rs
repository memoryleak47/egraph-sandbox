use crate::*;

impl EGraph {
    // creates a new eclass with slots "l.slots() cap r.slots()".
    pub fn union(&mut self, l: AppliedId, r: AppliedId) {
        // normalize inputs
        let l = self.normalize_applied_id_by_unionfind(l);
        let r = self.normalize_applied_id_by_unionfind(r);

        // early return, if union should not be made.
        if l == r { return; }

        if l.id == r.id {
            eprintln!("We reject self-unions for now!");
            return;
        };

        // make the slots fresh.
        let all_slots = l.slots().union(&r.slots()).copied().collect();
        let fresh_map = SlotMap::bijection_from_fresh_to(&all_slots).inverse();
        let l = l.apply_slotmap(&fresh_map);
        let r = r.apply_slotmap(&fresh_map);

        let slots: HashSet<Slot> = l.slots().intersection(&r.slots()).copied().collect();
        let c_id = self.alloc_eclass(&slots);

        for lr in [l, r] {
            self.merge_into_eclass(lr.id, c_id, &lr.m);
        }

        // rebuild the egraph invariants
        self.fix_new_redundant_slots();
        self.fix_shape_collisions();
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

        // the ENode (identified by its shape) only has a strict subset of the slots, relative to its eclass.
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

        fn mark_redundant_slot(eg: &mut EGraph, id: Id, sh: Shape) {
            let bij = &eg.classes[&id].nodes[&sh];

            // the new _smaller_ set of slots.
            let slots: HashSet<Slot> = eg.classes[&id].slots.intersection(&bij.values()).copied().collect();

            let c = eg.alloc_eclass(&slots);

            let identity = SlotMap::identity(&slots);
            eg.merge_into_eclass(id, c, &identity);
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

        self.normalize_enode_usages();
    }

    // normalizes all ENodes that come up in the EGraph using the unionfind.
    fn normalize_enode_usages(&mut self) {
        for (i, c) in self.classes.clone() {
            let mut new_nodes = HashMap::new();
            for (sh, bij) in c.nodes {
                let n = sh.apply_slotmap(&bij);
                let n = self.normalize_enode_by_unionfind(&n);
                let (sh, bij) = n.shape();
                new_nodes.insert(sh, bij);
            }
            self.classes.get_mut(&i).unwrap().nodes = new_nodes;
        }
    }
}
