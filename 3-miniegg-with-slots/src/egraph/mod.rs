use crate::*;

mod find;
pub use find::*;

mod add;
pub use add::*;

mod union;
pub use union::*;

#[derive(Clone, Debug)]
pub struct EClass<L: Language> {
    // The set of equivalent ENodes that make up this eclass.
    // for (sh, bij) in nodes; sh.apply_slotmap(bij) represents the actual ENode.
    nodes: HashMap<L, Bijection>,

    // All other slots are considered "redundant" (or they have to be qualified by a ENode::Lam).
    // Should not contain Slot(0).
    slots: HashSet<Slot>,

    // Shows which Shapes refer to this EClass.
    usages: HashSet<L>,

    // Expresses the self-symmetries of this e-class.
    group: Group,
}

// invariants:
// 1. If two ENodes (that are in the EGraph) have equal .shape(), they have to be in the same eclass.
// 2. enode.slots() is always a superset of c.slots, if enode is within the eclass c.
//    if ENode::Lam(si) = enode, then we require i to not be in c.slots.
//    In practice, si will always be Slot(0).
// 3. AppliedId::m is always a bijection. (eg. c1(s0, s1, s0) is illegal!)
//    AppliedId::m also always has the same keys as the class expects slots.
// 4. Slot(0) should not be in EClass::slots of any class.
#[derive(Debug)]
pub struct EGraph<L: Language> {
    // an entry (l, r(sa, sb)) in unionfind corresponds to the equality l(s0, s1, s2) = r(sa, sb), where sa, sb in {s0, s1, s2}.
    // normalizes the eclass.
    // Each Id i that is an output of the unionfind itself has unionfind[i] = (i, identity()).
    unionfind: Unionfind,

    // if a class does't have unionfind[x].id = x, then it doesn't contain nodes / usages.
    // It's "shallow" if you will.
    classes: HashMap<Id, EClass<L>>,

    // For each shape contained in the EGraph, maps to the EClass that contains it.
    hashcons: HashMap<L, Id>,
}

impl<L: Language> EGraph<L> {
    pub fn new() -> Self {
        EGraph {
            unionfind: Default::default(),
            classes: Default::default(),
            hashcons: Default::default(),
        }
    }

    pub fn slots(&self, id: Id) -> HashSet<Slot> {
        self.classes[&id].slots.clone()
    }

    #[track_caller]
    pub fn mk_applied_id(&self, i: Id, m: SlotMap) -> AppliedId {
        let app_id = AppliedId::new(i, m);

        if CHECKS {
            self.check_applied_id(&app_id);
        }

        app_id
    }

    #[track_caller]
    pub fn mk_identity_applied_id(&self, i: Id) -> AppliedId {
        self.mk_applied_id(i, SlotMap::identity(&self.classes[&i].slots))
    }

    #[track_caller]
    pub fn check_applied_id(&self, app_id: &AppliedId) {
        app_id.check();
        assert_eq!(self.classes[&app_id.id].slots, app_id.m.keys());
    }

    pub fn ids(&self) -> Vec<Id> {
        self.unionfind.iter()
                       .filter(|(x, y)| x == &y.id)
                       .map(|(x, _)| x)
                       .collect()
    }

    // TODO For non-normalized inputs i, the slots in the output will definitely be wrong.
    // if x in enodes(i), then I'd expect x.slots() superset slots(i).
    pub fn enodes(&self, i: Id) -> HashSet<L> {
        let i = self.find_id(i);
        self.classes[&i].nodes.iter().map(|(x, y)| x.apply_slotmap(y)).collect()
    }

    // Generates fresh slots for redundant slots.
    pub fn enodes_applied(&self, i: &AppliedId) -> HashSet<L> {
        let i = self.find_applied_id(i);

        let mut out = HashSet::default();
        for x in self.enodes(i.id) {
            // This is necessary, as i.slots() might collide with the private/redundant slots of our e-nodes.
            let set: HashSet<_> = x.all_slot_occurences()
                                   .into_iter()
                                   .collect::<HashSet<_>>()
                                   .difference(&self.classes[&i.id].slots)
                                   .copied()
                                   .collect();
            let x = x.refresh_slots(set);

            let red = &x.slots() - &i.m.keys();
            let fbij = SlotMap::bijection_from_fresh_to(&red);
            let m = fbij.inverse().union(&i.m);
            out.insert(x.apply_slotmap(&m));
        }

        if CHECKS {
            for x in &out {
                assert_eq!(self.lookup(x).unwrap(), i);
            }
        }

        out
    }

    // number of enodes in the egraph.
    pub fn total_number_of_nodes(&self) -> usize {
        self.hashcons.len()
    }

    pub fn eq(&self, a: &AppliedId, b: &AppliedId) -> bool {
        let a = self.find_applied_id(a);
        let b = self.find_applied_id(b);

        if CHECKS {
            self.check_applied_id(&a);
            self.check_applied_id(&b);
        }

        if a.id != b.id { return false; }
        if a.m.values() != b.m.values() { return false; }
        let id = a.id;

        let perm = a.m.compose(&b.m.inverse());
        if CHECKS {
            assert!(perm.is_perm());
            assert_eq!(&perm.values(), &self.classes[&id].slots);
        }

        self.classes[&id].group.contains(&perm)
    }

    pub fn check(&self) {
        // Checks whether the hashcons / usages are correct.
        // And also checks that each Shape comes up in at most one EClass!
        let mut hashcons = HashMap::default();
        let mut usages = HashMap::default();

        for (i, _) in &self.classes {
            usages.insert(*i, HashSet::default());
        }

        for (i, c) in &self.classes {
            for sh in c.nodes.keys() {
                assert!(!hashcons.contains_key(sh));
                hashcons.insert(sh.clone(), *i);

                for ref_id in sh.ids() {
                    usages.get_mut(&ref_id).unwrap()
                          .insert(sh.clone());
                }
            }
        }

        assert_eq!(hashcons, self.hashcons);
        for (i, c) in &self.classes {
            assert_eq!(usages[&i], c.usages);
        }

        // check that self.classes contains exactly these classes which point to themselves in the unionfind.
        let all_keys = self.unionfind.iter().map(|(x, _)| x).collect::<HashSet<_>>();
        let all_values = self.unionfind.iter().map(|(_, x)| x.id).collect::<HashSet<_>>();
        let all_classes = self.classes.keys().copied().collect::<HashSet<_>>();
        let all: HashSet<Id> = &(&all_keys | &all_values) | &all_classes;
        for i in all {
            // if they point to themselves, they should do it using the identity.
            if self.is_alive(i) {
                assert_eq!(self.unionfind.get(i), self.mk_identity_applied_id(i));
            } else {
                assert!(self.classes[&i].nodes.is_empty());
                assert!(self.classes[&i].usages.is_empty());
            }
        }

        // check that no EClass has Slot(0) in its API.
        for (_, c) in &self.classes {
            assert!(!c.slots.contains(&Slot::new(0)));
        }

        // Check that the Unionfind has valid AppliedIds.
        for (_, app_id) in self.unionfind.iter() {
            check_internal_applied_id::<L>(self, &app_id);
        }

        // Check that all ENodes are valid.
        for (_, c) in &self.classes {
            for (sh, bij) in &c.nodes {
                let real = sh.apply_slotmap(bij);
                assert!(real.slots().is_superset(&c.slots));

                assert_eq!((sh.clone(), bij.clone()), self.shape(&real));

                for x in real.applied_id_occurences() {
                    check_internal_applied_id::<L>(self, &x);
                }
            }
        }

        fn check_internal_applied_id<L: Language>(eg: &EGraph<L>, app_id: &AppliedId) {
            // 1. the app_id needs to be normalized!
            let y = eg.find_applied_id(app_id);
            assert_eq!(app_id, &y);

            // 2. It needs to have exactly the same slots as the underlying EClass.
            assert_eq!(&app_id.m.keys(), &eg.classes[&app_id.id].slots);
        }
    }

    fn is_alive(&self, i: Id) -> bool {
        self.find_id(i) == i
    }

    // refreshes all internal slots of l.
    pub fn refresh_internals(&self, l: &L) -> L {
        let i = self.lookup(l).unwrap();
        l.refresh_internals(i.slots())
    }

    // converts l to its class normal form, so that calling lookup on it yields the identity AppliedId.
    pub fn class_nf(&self, l: &L) -> L {
        let l = self.refresh_internals(l);
        let i = self.lookup(&l).unwrap();
        let l = l.apply_slotmap(&i.m);

        if CHECKS {
            assert!(self.lookup(&l).unwrap().m.iter().all(|(x, y)| x == y));
        }

        l
    }

    pub fn dump(&self) {
        println!("");

        let mut v: Vec<(&Id, &EClass<L>)> = self.classes.iter().collect();
        v.sort_by_key(|(x, _)| *x);

        for (i, c) in v {
            if !self.is_alive(*i) { continue; }

            let slot_str = c.slots.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", ");
            println!("{:?}({}):", i, &slot_str);
            for (sh, bij) in &c.nodes {
                let n = sh.apply_slotmap(bij);
                println!(" - {:?}", n);
            }
            for p in &c.group.generators() {
                println!(" -- {:?}", p);
            }
        }
        println!("");
    }

    // The resulting e-nodes are written as they exist in the e-class.
    pub fn usages(&self, i: Id) -> Vec<L> {
        let mut out = Vec::new();
        for x in &self.classes[&i].usages {
            let j = self.lookup(x).unwrap().id;
            let bij = &self.classes[&j].nodes[&x];
            let x = x.apply_slotmap(bij);
            out.push(x);
        }
        out
    }

    pub fn shape(&self, e: &L) -> (L, Bijection) {
        self.get_group_compatible_variants2(e)
            .iter()
            .map(|x| x.weak_shape())
            .min_by_key(|(x, _)| x.all_slot_occurences()).unwrap()
    }

    // for all AppliedIds that are contained in `enode`, permute their arguments as their groups allow.
    fn get_group_compatible_variants2(&self, enode: &L) -> HashSet<L> {
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

}
