use crate::*;

pub enum Child {
    AppliedId(AppliedId),
    Slot(Slot),
}

pub trait Language: Debug + Clone + Hash + Eq {
    // returns non-deduplicated lists of all occurences of these things, in order.
    fn all_slot_occurences_mut(&mut self) -> Vec<&mut Slot>;
    fn public_slot_occurences_mut(&mut self) -> Vec<&mut Slot>;
    fn applied_id_occurences_mut(&mut self) -> Vec<&mut AppliedId>;

    // for parsing and pretty-printing.
    fn to_op(&self) -> (String, Vec<Child>);
    fn from_op(op: &str, children: Vec<Child>) -> Option<Self>;

    #[track_caller]
    fn check(&self) {
        let mut c = self.clone();
        let all: HashSet<*mut Slot> = c.all_slot_occurences_mut().into_iter().map(|x| x as *mut Slot).collect();
        let public: HashSet<*mut Slot> = c.public_slot_occurences_mut().into_iter().map(|x| x as *mut Slot).collect();
        let private: HashSet<*mut Slot> = c.private_slot_occurences_mut().into_iter().map(|x| x as *mut Slot).collect();

        assert!(public.is_disjoint(&private));

        // This also catches errors, where different Slot-addresses have the same slot names. This also counts as a collision!
        let f = |x: Vec<Slot>| x.into_iter().collect::<HashSet<_>>();
        assert!(f(c.public_slot_occurences()).is_disjoint(&f(c.private_slot_occurences())));

        let all2: HashSet<*mut Slot> = public.union(&private).copied().collect();
        assert_eq!(all2, all);
    }


    // generated methods:

    fn private_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        let public = self.public_slot_occurences();
        let mut out = self.all_slot_occurences_mut();
        out.retain(|x| !public.contains(x));
        out
    }

    fn all_slot_occurences(&self) -> Vec<Slot> {
        self.clone().all_slot_occurences_mut().into_iter().map(|x| x.clone()).collect()
    }

    fn public_slot_occurences(&self) -> Vec<Slot> {
        self.clone().public_slot_occurences_mut().into_iter().map(|x| x.clone()).collect()
    }

    fn applied_id_occurences(&self) -> Vec<AppliedId> {
        self.clone().applied_id_occurences_mut().into_iter().map(|x| x.clone()).collect()
    }

    fn private_slot_occurences(&self) -> Vec<Slot> {
        self.clone().private_slot_occurences_mut().into_iter().map(|x| x.clone()).collect()
    }

    fn private_slots(&self) -> HashSet<Slot> {
        self.private_slot_occurences().into_iter().collect()
    }

    fn map_applied_ids(&self, f: impl Fn(AppliedId) -> AppliedId) -> Self {
        let mut c = self.clone();
        for x in c.applied_id_occurences_mut() {
            *x = f(x.clone());
        }
        c
    }

    // TODO m.values() might collide with your private slot names.
    // Should we rename our private slots to be safe?
    fn apply_slotmap_partial(&self, m: &SlotMap) -> Self {
        let prv = self.private_slots();

        let mut c = self.clone();
        for x in c.public_slot_occurences_mut() {
            let y = m[*x];

            // If y collides with a private slot, we have a problem.
            if SMALL_CHECKS {
                assert!(!prv.contains(&y));
            }

            *x = y;
        }
        c
    }

    #[track_caller]
    fn apply_slotmap(&self, m: &SlotMap) -> Self {
        if SMALL_CHECKS {
            assert!(m.keys().is_superset(&self.slots()), "Language::apply_slotmap: The SlotMap doesn't map all free slots!");
        }
        self.apply_slotmap_partial(m)
    }

    fn slot_occurences(&self) -> Vec<Slot> {
        self.public_slot_occurences()
    }

    fn slot_order(&self) -> Vec<Slot> { firsts(self.slot_occurences()) }
    fn slots(&self) -> HashSet<Slot> { as_set(self.slot_occurences()) }

    fn ids(&self) -> Vec<Id> {
        self.applied_id_occurences().into_iter().map(|x| x.id).collect()
    }

    // let n.weak_shape() = (sh, bij); then
    // - sh.apply_slotmap(bij) is equivalent to n (excluding lambda variable renames)
    // - bij.slots() == n.slots(). Note that these would also include redundant slots.
    // - sh is the lexicographically lowest equivalent version of n, reachable by bijective renaming of slots (including redundant ones).
    fn weak_shape(&self) -> (Self, Bijection) {
        let mut c = self.clone();
        let mut m = SlotMap::new();
        let mut i = 0;

        for x in c.all_slot_occurences_mut() {
            let x_val = *x;
            if !m.contains_key(x_val) {
                let new_slot = Slot::new(i);
                i += 1;

                m.insert(x_val, new_slot);
            }

            *x = m[x_val];
        }

        let m = m.inverse();

        let public = c.slots();
        let m: SlotMap = m.iter().filter(|(x, _)| public.contains(x)).collect();

        (c, m)
    }

    fn refresh_private(&self) -> Self {
        let mut c = self.clone();
        let prv: HashSet<Slot> = c.private_slot_occurences().into_iter().collect();
        let fresh = SlotMap::bijection_from_fresh_to(&prv).inverse();
        for x in c.private_slot_occurences_mut() {
            *x = fresh[*x];
        }
        c
    }

    fn refresh_slots(&self, set: HashSet<Slot>) -> Self {
        let mut c = self.clone();
        let fresh = SlotMap::bijection_from_fresh_to(&set).inverse();
        for x in c.all_slot_occurences_mut() {
            if set.contains(x) {
                *x = fresh[*x];
            }
        }
        c
    }

    // refreshes private and redundant slots.
    // The public slots are given by `public`.
    fn refresh_internals(&self, public: HashSet<Slot>) -> Self {
        let mut c = self.clone();
        let internals = &c.all_slot_occurences().into_iter().collect::<HashSet<_>>() - &public;
        let fresh = SlotMap::bijection_from_fresh_to(&internals).inverse();
        for x in c.all_slot_occurences_mut() {
            if internals.contains(x) {
                *x = fresh[*x];
            }
        }
        c
    }
}

// sorts as_set(v) by their first usage in v.
pub fn firsts(v: Vec<Slot>) -> Vec<Slot> {
    let mut out = Vec::new();
    for x in v {
        if !out.contains(&x) {
            out.push(x);
        }
    }
    out
}

pub fn as_set(v: Vec<Slot>) -> HashSet<Slot> {
    v.into_iter().collect()
}


// make their inner private variables named the same.
// assumes that a and b have the same e-node operator, but they might potentially reference different applied ids.
pub fn unify_private_slots<L: Language>(a: &L, b: &L) -> (L, L) {
    let mut a = a.clone();
    let mut b = b.clone();

    let null_a = nullify_app_ids(&a);
    let null_b = nullify_app_ids(&b);

    // check that a and b have the same operator.
    if SMALL_CHECKS {
        assert_eq!(null_a.weak_shape().0, null_b.weak_shape().0);
    }

    // construct amap & bmap, mapping their private slots to a common fresh slot name.
    let mut amap = HashMap::default();
    let mut bmap = HashMap::default();

    let apriv = null_a.private_slot_occurences().into_iter();
    let bpriv = null_b.private_slot_occurences().into_iter();

    for (ap, bp) in apriv.zip(bpriv) {
        match (amap.get(&ap), bmap.get(&bp)) {
            (Some(xa), Some(xb)) => assert_eq!(xa, xb),
            (None, None) => {
                let s = Slot::fresh();
                amap.insert(ap, s);
                bmap.insert(bp, s);
            },
            _ => panic!("amap and bmap should match the same things all the time!"),
        }
    }

    // Note that a and b might have different amounts of private slot occurences, because they might reference different applied ids.
    for ap in a.private_slot_occurences_mut() {
        *ap = amap[ap];
    }

    for bp in b.private_slot_occurences_mut() {
        *bp = bmap[bp];
    }

    (a, b)
}
