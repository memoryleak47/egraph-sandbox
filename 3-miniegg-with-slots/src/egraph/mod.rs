use crate::*;

mod find;
pub use find::*;

mod add;
pub use add::*;

mod union;
pub use union::*;

mod expl;
pub use expl::*;

use std::sync::Mutex;

/// Each E-Class can be understood "semantically" or "syntactically":
/// - semantically means that it respects the equations already in the e-graph, and hence doesn't differentiate between equal things.
/// - syntactically means that it only talks about the single representative term associated to each E-Class, recursively obtainable using syn_enode.
#[derive(Clone, Debug)]
pub struct EClass<L: Language> {
    // The set of equivalent ENodes that make up this eclass.
    // for (sh, bij) in nodes; sh.apply_slotmap(bij) represents the actual ENode.
    nodes: HashMap<L, (Bijection, /*remembers the original AppliedId, where this came from*/ AppliedId)>,

    // All other slots are considered "redundant" (or they have to be qualified by a ENode::Lam).
    // Should not contain Slot(0).
    slots: HashSet<Slot>,

    // Shows which Shapes refer to this EClass.
    usages: HashSet<L>,

    // Expresses the self-symmetries of this e-class.
    group: Group<ProvenPerm>,

    syn_enode: Option<L>,
    redundancy_proof: Option<ProvenEq>,
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

    // We use mutex to allow for inter mutability, so that find(&self) can do path compression.
    unionfind: Mutex<Vec<(AppliedId, ProvenEq)>>,

    // if a class does't have unionfind[x].id = x, then it doesn't contain nodes / usages.
    // It's "shallow" if you will.
    classes: HashMap<Id, EClass<L>>,

    // For each shape contained in the EGraph, maps to the EClass that contains it.
    hashcons: HashMap<L, Id>,

    // For each (syn_slotset applied) non-normalized (i.e. "syntactic") weak shape, find the e-class who has this as syn_enode.
    syn_hashcons: HashMap<L, AppliedId>,
}

impl<L: Language> EGraph<L> {
    pub fn new() -> Self {
        EGraph {
            unionfind: Default::default(),
            classes: Default::default(),
            hashcons: Default::default(),
            syn_hashcons: Default::default(),
        }
    }

    pub fn slots(&self, id: Id) -> HashSet<Slot> {
        self.classes[&id].slots.clone()
    }

    pub fn syn_slots(&self, id: Id) -> HashSet<Slot> {
        self.classes[&id].syn_enode.as_ref().unwrap().slots()
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
        self.unionfind_iter()
                       .filter(|(x, y)| x == &y.id)
                       .map(|(x, _)| x)
                       .collect()
    }

    // TODO For non-normalized inputs i, the slots in the output will definitely be wrong.
    // if x in enodes(i), then I'd expect x.slots() superset slots(i).
    pub fn enodes(&self, i: Id) -> HashSet<L> {
        let i = self.find_id(i);
        self.classes[&i].nodes.iter().map(|(x, (y, _))| x.apply_slotmap(y)).collect()
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
                assert!(self.eq(&self.lookup(x).unwrap(), &i));
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

        // TODO fix ::null()
        let proven_perm = ProvenPerm(perm, ProvenEqRaw::null());
        self.classes[&id].group.contains(&proven_perm)
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
        let all_keys = self.unionfind_iter().map(|(x, _)| x).collect::<HashSet<_>>();
        let all_values = self.unionfind_iter().map(|(_, x)| x.id).collect::<HashSet<_>>();
        let all_classes = self.classes.keys().copied().collect::<HashSet<_>>();
        let all: HashSet<Id> = &(&all_keys | &all_values) | &all_classes;
        for i in all {
            // if they point to themselves, they should do it using the identity.
            if self.is_alive(i) {
                assert_eq!(self.unionfind_get(i), self.mk_identity_applied_id(i));
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
        for (_, app_id) in self.unionfind_iter() {
            check_internal_applied_id::<L>(self, &self.semify_app_id(app_id));
        }

        // Check that all ENodes are valid.
        for (_, c) in &self.classes {
            for (sh, (bij, _)) in &c.nodes {
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
            let slot_str = c.slots.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", ");
            println!("\n{:?}({}):", i, &slot_str);

            if let Some(syn_n) = &c.syn_enode {
                println!(">> {syn_n:?}");
            }

            for (sh, (bij, app_id)) in &c.nodes {
                let n = sh.apply_slotmap(bij);
                println!(" - {n:?}    [originally {app_id:?}]");
            }
            for ProvenPerm(p, _) in &c.group.generators() {
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
            let bij = &self.classes[&j].nodes[&x].0;
            let x = x.apply_slotmap(bij);
            out.push(x);
        }
        out
    }

    pub fn shape(&self, e: &L) -> (L, Bijection) {
        self.proven_shape(e).0
    }

    pub fn proven_shape(&self, e: &L) -> ((L, Bijection), Vec<ProvenEq>) {
        let (e, v1) = self.proven_find_enode(e);
        let (t, v2) = self.proven_get_group_compatible_variants(&e)
            .into_iter()
            .map(|(x, prfs)| (x.weak_shape(), prfs))
            .min_by_key(|((x, _), _)| x.all_slot_occurences()).unwrap();

        let mut out: Vec<ProvenEq> = Vec::new();

        assert_eq!(v1.len(), e.applied_id_occurences().len());
        assert_eq!(v2.len(), e.applied_id_occurences().len());

        let v1 = v1.into_iter();
        let v2 = v2.into_iter();

        for (l, r) in v1.zip(v2) {
            out.push(prove_transitivity(l, r));
        }

        (t, out)
    }

    fn refl_proof(&self, i: Id) -> ProvenEq {
        let syn_slots = self.syn_slots(i);
        let identity = SlotMap::identity(&syn_slots);
        let app_id = AppliedId::new(i, identity);;
        prove_reflexivity(&app_id)
    }

    fn apply_proven_perm(&self, (x, x_prf): (AppliedId, ProvenEq), ProvenPerm(y, y_prf): &ProvenPerm) -> (AppliedId, ProvenEq) {
        let mut x = x;
        let mut x_prf = x_prf;

        // TODO these seem to be in different order. why is that?
        x = self.mk_applied_id(x.id, y.compose(&x.m));
        x_prf = prove_transitivity(x_prf, y_prf.clone());
        (x, x_prf)
    }

    // for all AppliedIds that are contained in `enode`, permute their arguments as their groups allow.
    // TODO every usage of this function hurts performance drastically. Which of them can I eliminate?
    pub fn proven_get_group_compatible_variants(&self, enode: &L) -> HashSet<(L, Vec<ProvenEq>)> {
        let mut s = HashSet::default();

        let n = enode.applied_id_occurences().len();

        let mut init = Vec::new();
        for x in enode.applied_id_occurences() {
            init.push(self.refl_proof(x.id));
        }

        s.insert((enode.clone(), init));

        for (i, app_id) in enode.applied_id_occurences().iter().enumerate() {
            let grp_perms = self.classes[&app_id.id].group.all_perms();
            let mut next = HashSet::default();
            for (x, x_prfs) in s {
                for proven_perm in &grp_perms {
                    let x_i = x.applied_id_occurences()[i].clone();
                    let x_prfs_i = x_prfs[i].clone();
                    let (app_id, prf) = self.apply_proven_perm((x_i, x_prfs_i), proven_perm);

                    let mut x2 = x.clone();
                    *x2.applied_id_occurences_mut()[i] = app_id;

                    let mut x_prfs2 = x_prfs.clone();
                    x_prfs2[i] = prf;

                    next.insert((x2, x_prfs2));
                }
            }
            s = next;
        }

        s
    }

    pub fn get_group_compatible_variants(&self, enode: &L) -> HashSet<L> {
        self.proven_get_group_compatible_variants(enode).into_iter().map(|(x, _)| x).collect()
    }

    pub fn get_group_compatible_weak_variants(&self, enode: &L) -> HashSet<L> {
        let set = self.get_group_compatible_variants(enode);
        let mut shapes = HashSet::default();
        let mut out = HashSet::default();

        for x in set {
            let (sh, _) = x.weak_shape();
            if shapes.contains(&sh) { continue; }
            shapes.insert(sh);
            out.insert(x);
        }

        out
    }

    pub fn synify_app_id(&self, app: AppliedId) -> AppliedId {
        let mut app = app;
        for s in self.syn_slots(app.id) {
            if !app.m.contains_key(s) {
                app.m.insert(s, Slot::fresh());
            }
        }
        app
    }

    pub fn synify_enode(&self, enode: L) -> L {
        enode.map_applied_ids(|app| self.synify_app_id(app))
    }

    pub fn semify_app_id(&self, app: AppliedId) -> AppliedId {
        let slots = self.slots(app.id);

        let mut app = app;
        for k in app.m.keys() {
            if !slots.contains(&k) {
                app.m.remove(k);
            }
        }
        app
    }

    pub fn semify_enode(&self, enode: L) -> L {
        enode.map_applied_ids(|app| self.semify_app_id(app))
    }

    pub fn get_syn_expr(&self, i: &AppliedId) -> RecExpr<L> {
        let enode = self.get_syn_node(i);
        let cs = enode.applied_id_occurences()
                      .iter()
                      .map(|x| self.get_syn_expr(x))
                      .collect();
        RecExpr {
            node: nullify_app_ids(&enode),
            children: cs,
        }
    }
}
