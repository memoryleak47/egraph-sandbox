use crate::*;

impl<L: Language> EGraph<L> {
    // mk_applied_id & friends.
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
        let set = app_id.m.keys();
        let mini = self.slots(app_id.id);
        let maxi = self.syn_slots(app_id.id);
        assert!(set.is_subset(&maxi), "checking AppliedId failed: Too large key-set, {app_id:?}");
        assert!(set.is_superset(&mini), "checking AppliedId failed: Too small key-set, {app_id:?}");
    }


    // mk_sem_applied_id & friends.
    #[track_caller]
    pub fn mk_sem_applied_id(&self, i: Id, m: SlotMap) -> AppliedId {
        let app_id = AppliedId::new(i, m);

        if CHECKS {
            self.check_sem_applied_id(&app_id);
        }

        app_id
    }

    #[track_caller]
    pub fn mk_identity_sem_applied_id(&self, i: Id) -> AppliedId {
        self.mk_sem_applied_id(i, SlotMap::identity(&self.slots(i)))
    }

    #[track_caller]
    pub fn check_sem_applied_id(&self, app_id: &AppliedId) {
        app_id.check();
        assert_eq!(self.slots(app_id.id), app_id.m.keys(), "checking sem AppliedId failed: Wrong key-set, {app_id:?}");
    }

    // mk_syn_applied_id & friends.
    #[track_caller]
    pub fn mk_syn_applied_id(&self, i: Id, m: SlotMap) -> AppliedId {
        let app_id = AppliedId::new(i, m);

        if CHECKS {
            self.check_syn_applied_id(&app_id);
        }

        app_id
    }


    #[track_caller]
    pub fn mk_identity_syn_applied_id(&self, i: Id) -> AppliedId {
        self.mk_syn_applied_id(i, SlotMap::identity(&self.syn_slots(i)))
    }


    #[track_caller]
    pub fn check_syn_applied_id(&self, app_id: &AppliedId) {
        app_id.check();
        assert_eq!(self.syn_slots(app_id.id), app_id.m.keys(), "checking syn AppliedId failed: Wrong key-set, {app_id:?}");
    }


    pub fn check_groups(&self) {
        for (i, c) in &self.classes {
            for p in c.group.all_perms() {
                p.check();
            }
        }
    }

    pub fn check(&self) {
        self.check_groups();

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
}
