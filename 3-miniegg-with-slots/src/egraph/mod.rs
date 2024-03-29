use crate::*;

mod add;
pub use add::*;

mod union;
pub use union::*;

#[derive(Clone, Debug)]
pub struct EClass {
    // The set of equivalent ENodes that make up this eclass.
    // for (sh, bij) in nodes; sh.apply_slotmap(bij) represents the actual ENode.
    nodes: HashMap<Shape, Bijection>,

    // All other slots are considered "redundant" (or they have to be qualified by a ENode::Lam).
    // Should not contain Slot(0).
    slots: HashSet<Slot>,

    // Shows which Shapes refer to this EClass.
    usages: HashSet<Shape>,
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
pub struct EGraph {
    // an entry (l, r(sa, sb)) in unionfind corresponds to the equality l(s0, s1, s2) = r(sa, sb), where sa, sb in {s0, s1, s2}.
    // normalizes the eclass.
    // Each Id i that is an output of the unionfind itself has unionfind[i] = (i, identity()).
    unionfind: HashMap<Id, AppliedId>,

    // if a class does't have unionfind[x].id = x, then it doesn't contain nodes / usages.
    // It's "shallow" if you will.
    classes: HashMap<Id, EClass>,

    // For each shape contained in the EGraph, maps to the EClass that contains it.
    hashcons: HashMap<Shape, Id>,
}

impl EGraph {
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

    pub fn find_enode(&self, enode: &ENode) -> ENode {
        enode.map_applied_ids(|x| self.find_applied_id(x))
    }

    #[track_caller]
    pub fn mk_applied_id(&self, i: Id, m: SlotMap) -> AppliedId {
        let app_id = AppliedId::new(i, m);

        self.check_applied_id(&app_id);

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

    // normalize i.id
    //
    // Example 1:
    // 'find(c1(s10, s11)) = c2(s11, s10)', where 'c1(s0, s1) -> c2(s1, s0)' in unionfind.
    //
    // Example 2:
    // 'find(c1(s3, s7, s8)) = c2(s8, s7)', where 'c1(s0, s1, s2) -> c2(s2, s1)' in unionfind,
    pub fn find_applied_id(&self, i: AppliedId) -> AppliedId {
        let a = &self.unionfind[&i.id];

        // I = self.slots(i.id);
        // A = self.slots(a.id);
        // i.m   :: I -> X
        // a.m   :: A -> I
        // out.m :: A -> X

        self.mk_applied_id(
            a.id,
            a.m.compose_partial(&i.m), // This is partial if `i.id` had redundant slots.
        )
    }

    pub fn find_id(&self, i: Id) -> Id {
        let i = self.unionfind[&i].id;
        // TODO what's the usecase for this?
        // Don't we want a consistent AppliedId API?
        assert!(self.classes[&i].slots.is_empty());

        i
    }

    pub fn ids(&self) -> Vec<Id> {
        self.unionfind.iter()
                       .filter(|(x, y)| x == &&y.id)
                       .map(|(x, _)| *x)
                       .collect()
    }

    // TODO For non-normalized inputs i, the slots in the output will definitely be wrong.
    // if x in enodes(i), then I'd expect x.slots() superset slots(i).
    pub fn enodes(&self, i: Id) -> HashSet<ENode> {
        let i = self.unionfind[&i].id;
        self.classes[&i].nodes.iter().map(|(x, y)| x.apply_slotmap(y)).collect()
    }

    // Generates fresh slots for redundant slots.
    pub fn enodes_applied(&self, i: &AppliedId) -> HashSet<ENode> {
        let i = self.find_applied_id(i.clone());

        let mut out = HashSet::default();
        for x in self.enodes(i.id) {
            let red = &x.slots() - &i.m.keys();
            let fbij = SlotMap::bijection_from_fresh_to(&red);
            let m = fbij.inverse().union(&i.m);
            out.insert(x.apply_slotmap(&m));
        }

        for x in &out {
            assert_eq!(self.lookup(x).unwrap(), i);
        }

        out
    }

    // number of enodes in the egraph.
    pub fn total_size(&self) -> usize {
        self.hashcons.len()
    }

    pub fn inv(&self) {
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
        let all: HashSet<&Id> = &(&self.unionfind.keys().collect::<HashSet<_>>() | &self.unionfind.values().map(|x| &x.id).collect::<HashSet<_>>()) | &self.classes.keys().collect::<HashSet<_>>();
        for i in all {
            let alive = self.classes[i].is_alive();
            let alive2 = self.unionfind[i].id == *i;
            assert_eq!(alive, alive2);

            // if they point to themselves, they should do it using the identity.
            if alive {
                let slots = &self.classes[i].slots;
                assert_eq!(self.unionfind[i].m, SlotMap::identity(slots));
            } else {
                assert!(self.classes[i].nodes.is_empty());
                assert!(self.classes[i].usages.is_empty());
            }
        }

        // check that no EClass has Slot(0) in its API.
        for (i, c) in &self.classes {
            assert!(!c.slots.contains(&Slot(0)));
        }

        // Check that the Unionfind has valid AppliedIds.
        for (_, app_id) in &self.unionfind {
            inv_internal_applied_id(self, app_id);
        }

        // Check that all ENodes are valid.
        for (i, c) in &self.classes {
            for (sh, bij) in &c.nodes {
                let real = sh.apply_slotmap(bij);
                assert!(real.slots().is_superset(&c.slots));

                assert_eq!((sh.clone(), bij.clone()), real.shape());

                match real {
                    ENode::Var(x) => {
                        assert_eq!(&singleton_set(x), &c.slots)
                    },
                    ENode::App(l, r) => {
                        inv_internal_applied_id(self, &l);
                        inv_internal_applied_id(self, &r);
                    }
                    ENode::Lam(x, b) => {
                        assert_eq!(x, Slot(0));

                        inv_internal_applied_id(self, &b);
                    }
                }
            }
        }

        fn inv_internal_applied_id(eg: &EGraph, app_id: &AppliedId) {
            // 1. the app_id needs to be normalized!
            let y = eg.find_applied_id(app_id.clone());
            assert_eq!(app_id, &y);

            // 2. It needs to have exactly the same slots as the underlying EClass.
            assert_eq!(&app_id.m.keys(), &eg.classes[&app_id.id].slots);
        }
    }

    pub fn dump(&self) {
        println!("");
        for (i, c) in &self.classes {
            if !c.is_alive() { continue; }

            let slot_str = c.slots.iter().map(|x| format!("s{}", x.0)).collect::<Vec<_>>().join(", ");
            println!("{:?}({}):", i, &slot_str);
            for (sh, bij) in &c.nodes {
                let n = sh.apply_slotmap(bij);
                println!(" - {:?}", n);
            }
        }
        println!("");
    }
}

impl EClass {
    fn is_alive(&self) -> bool {
        !self.nodes.is_empty()
    }
}
