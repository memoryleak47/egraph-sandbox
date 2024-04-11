use crate::*;

pub trait Language: Debug + Clone + Hash + Eq {
    fn discr(&self) -> u32;

    // returns non-deduplicated lists of all occurences of these things, in order.
    fn all_slot_occurences_mut(&mut self) -> Vec<&mut Slot>;
    fn public_slot_occurences_mut(&mut self) -> Vec<&mut Slot>;
    fn applied_id_occurences_mut(&mut self) -> Vec<&mut AppliedId>;


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

            assert!(!prv.contains(x));

            *x = y;
        }
        c
    }

    #[track_caller]
    fn apply_slotmap(&self, m: &SlotMap) -> Self {
        assert!(m.keys().is_superset(&self.slots()), "Language::apply_slotmap: The SlotMap doesn't map all free slots!");
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

    // let n.shape() = (sh, bij); then
    // - sh.apply_slotmap(bij) is equivalent to n (excluding lambda variable renames)
    // - bij.slots() == n.slots(). Note that these would also include redundant slots.
    // - sh is the lexicographically lowest equivalent version of n, reachable by bijective renaming of slots (including redundant ones).
    fn shape(&self) -> (Self, Bijection) {
        let mut c = self.clone();
        let mut m = SlotMap::new();
        let mut i = 0;

        for x in c.all_slot_occurences_mut() {
            let x_val = *x;
            if !m.contains_key(x_val) {
                let new_slot = Slot(i);
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
