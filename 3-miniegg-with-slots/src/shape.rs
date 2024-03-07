use crate::*;

// an ENode that has been reduced to its shape.
pub type Shape = ENode;

impl EGraph {
    // let eg.shape(n) = (x, y); then
    // - x.apply_slotmap(y) is equivalent to n (excluding lambda variable renames)
    // - y.slots() == n.slots(). Note that these would also include redundant slots.
    // - x is the lexicographically lowest equivalent version of n, reachable by bijective renaming of slots and re-ordering of AppliedId-args.
    // - Note that y is not normalized! There are multiple possible outputs for y, depending on the symmetries of the EClass containing this shape.
    //
    // For two ENodes n1, n2 that only differentiate each other by
    // (1) the names of their public slots (including redundant slots) and lambda slots, and
    // (2) the order of their AppliedId arguments within the boundaries of their corresponding permutation groups;
    // then self.shape(n1).0 == self.shape(n2).0
    pub fn shape(&self, n: &ENode) -> (Shape, Bijection) {
        let n = self.normalize_enode_by_unionfind(n);

        #[allow(non_snake_case)]
        let N: HashSet<Slot> = n.slots();

        match n {
            ENode::Var(s) => {
                let s0 = Slot(0);
                let l = Shape::Var(s0);
                let r = Bijection::from([(s0, s)]);

                (l, r)
            },
            ENode::Lam(s, x) => {
                let s0 = Slot(0);
                let mut r = Bijection::new();
                r.insert(s0, s);

                for sx in x.m.values_vec() {
                    let next = Slot(r.len());
                    r.insert(next, sx);
                }

                let l = Shape::Lam(s0, x.apply_slotmap(&r.inverse()));
                r.remove(s0);

                (l, r)
            },
            ENode::App(l, r) => {
                let f = |x: &AppliedId| x.m.values_vec();

                let g = |x: &AppliedId| {
                    let grp = &self.classes[&x.id].perm_group;
                    let x = grp.rename_slots(&l.m);
                    let x = enhance_group(x, &N);
                    assert_eq!(&x.omega(), &N);
                    x
                };

                let lists = [f(&l), f(&r)];
                let perm_groups = [g(&l), g(&r)];

                let (perms, theta) = find_minimal_ordering(lists.clone(), perm_groups);

                let d = |i: usize /*: [0, 1]*/| -> AppliedId {
                    let slot_vec: Vec<Slot> = lists[i].iter()
                            .map(|x| perms[i][*x])
                            .map(|x| theta[x])
                            .collect();
                    let app_id: &AppliedId = [&l, &r][i];
                    let m: SlotMap = app_id.m.keys_vec().into_iter().zip(slot_vec.into_iter()).collect();

                    AppliedId::new(app_id.id, m)
                };

                let out = Shape::App(d(0), d(1));
                (out, theta.inverse())
            },
        }
    }
}

// extends the permutations of perm_group to be acting on the `superset`.
// All newly added Slots (delta) can be randomly mapped to each other.
fn enhance_group(perm_group: PermGroup, superset: &HashSet<Slot>) -> PermGroup {
    assert!(perm_group.omega().is_subset(superset));

    let delta: Vec<Slot> = superset.difference(&perm_group.omega()).cloned().collect();

    let mut generators = HashSet::new();
    for mut gen in perm_group.generators() {
        for d in &delta {
            gen.insert(*d, *d);
        }
        generators.insert(gen);
    }

    // add all possible permutations between the `delta` slots.
    if delta.len() > 1 {
        let n = delta.len();
        let s = |x| delta[x];

        let m = |f: fn(usize) -> usize| -> Perm {
            let mut slotmap = SlotMap::identity(&superset);
            for i in 0..n {
                slotmap.insert(s(i), s(f(i) % n));
            }

            slotmap
        };

        let shift = |x| x+1;
        let flip = |x| if x < 2 { 1 - x } else { x };

        generators.insert(m(shift));
        generators.insert(m(flip));
    }

    PermGroup::new(superset, generators)
}

// Might for a general Language be interesting for more than 2 inputs.
//
// N is a set of slots.
// Each lists[i] is an ordering of N.
// Each perm_groups[i] is a perm group over the set N.
// Slots from N have a lexicographical order _ < _.
//
// Returns (perms, theta)
// - where each perms[i] is a permutation: N -> N from perm_groups[i], and
// - theta is a mapping from N to s0..s|N|, s.t. we can derive
// - out_lists[i] = lists[i].map(|x| perms[i][x])
//                          .map(|x| theta[x])
// where `out_lists` (or equivalently out_lists[0] ++ out_lists[1]) is lexicographically minimal.
fn find_minimal_ordering(lists: [Vec<Slot>; 2], perm_groups: [PermGroup; 2]) -> ([Perm; 2], Bijection) {
    todo!()
}
