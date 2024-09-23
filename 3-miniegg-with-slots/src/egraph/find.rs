use crate::*;

impl<L: Language> EGraph<L> {
    // We lazily semify the entries, only when we encounter them.
    fn unionfind_semify_entry(&self, i: Id, entry: &mut (AppliedId, ProvenEq)) {
        // TODO update both sides of the equation using semify_app_id, if necessary.
        // This works by transitively chaining the shrink proofs to the entry.
    }

    fn unionfind_get_impl(&self, i: Id, map: &mut [(AppliedId, ProvenEq)]) -> (AppliedId, ProvenEq) {
        let entry = &mut map[i.0];
        self.unionfind_semify_entry(i, entry);

        let entry = entry.clone();

        if entry.0.id == i {
            return entry;
        }

        // entry.0.m :: slots(entry.0.id) -> slots(i)
        // entry_to_leader.0.m :: slots(leader) -> slots(entry.0.id)
        let entry_to_leader = self.unionfind_get_impl(entry.0.id, map);
        let new = (
            entry_to_leader.0.apply_slotmap(&entry.0.m),
            prove_transitivity(entry.1, entry_to_leader.1),
        );

        map[i.0] = new.clone();
        new
    }

    pub fn unionfind_set(&self, i: Id, app: AppliedId, proof: ProvenEq) {
        if CHECKS {
            assert_eq!(i, proof.l.id);
            assert_eq!(app.id, proof.r.id);
        }
        let mut lock = self.unionfind.lock().unwrap();
        if lock.len() == i.0 {
            lock.push((app, proof));
        } else {
            lock[i.0] = (app, proof);
        }
    }

    pub fn proven_unionfind_get(&self, i: Id) -> (AppliedId, ProvenEq) {
        let mut map = self.unionfind.lock().unwrap();
        self.unionfind_get_impl(i, &mut *map)
    }

    pub fn unionfind_get(&self, i: Id) -> AppliedId {
        self.proven_unionfind_get(i).0
    }

    pub fn unionfind_iter(&self) -> impl Iterator<Item=(Id, AppliedId)> {
        let mut map = self.unionfind.lock().unwrap();
        let mut out = Vec::new();

        for x in (0..map.len()).map(Id) {
            let y = self.unionfind_get_impl(x, &mut *map).0;
            out.push((x, y));
        }

        out.into_iter()
    }

    pub fn unionfind_len(&self) -> usize {
        self.unionfind.lock().unwrap().len()
    }

    pub fn find_enode(&self, enode: &L) -> L {
        self.proven_find_enode(enode).0
    }

    pub fn proven_find_enode(&self, enode: &L) -> (L, Vec<ProvenEq>) {
        let mut v = Vec::new();
        let out = enode.map_applied_ids(|x| {
            let (app, prf) = self.proven_find_applied_id(&x);
            v.push(prf);
            app
        });
        (out, v)
    }

    // normalize i.id
    //
    // Example 1:
    // 'find(c1(s10, s11)) = c2(s11, s10)', where 'c1(s0, s1) -> c2(s1, s0)' in unionfind.
    //
    // Example 2:
    // 'find(c1(s3, s7, s8)) = c2(s8, s7)', where 'c1(s0, s1, s2) -> c2(s2, s1)' in unionfind,
    pub fn find_applied_id(&self, i: &AppliedId) -> AppliedId {
        self.proven_find_applied_id(i).0
    }

    pub fn proven_find_applied_id(&self, i: &AppliedId) -> (AppliedId, ProvenEq) {
        let (a, prf) = self.proven_unionfind_get(i.id);

        // I = self.slots(i.id);
        // A = self.slots(a.id);
        // i.m   :: I -> X
        // a.m   :: A -> I
        // out.m :: A -> X

        let out = self.mk_applied_id(
            a.id,
            a.m.compose_partial(&i.m), // This is partial if `i.id` had redundant slots.
        );
        (out, prf)
    }

    pub fn find_id(&self, i: Id) -> Id {
        self.unionfind_get(i).id
    }
}
