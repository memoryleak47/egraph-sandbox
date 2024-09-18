use crate::*;

impl<L: Language> EGraph<L> {
    // We lazily semify the entries, only when we encounter them.
    fn unionfind_semify_entry(&self, i: Id, entry: &mut ProvenEq) {
        // TODO update both sides of the equation using semify_app_id, if necessary.
        // This works by transitively chaining the shrink proofs to the entry.
    }

    fn unionfind_get_impl(&self, i: Id, map: &mut [ProvenEq]) -> ProvenEq {
        let entry = &mut map[i.0];
        self.unionfind_semify_entry(i, entry);

        let entry = entry.clone();

        if entry.r.id == i {
            return entry;
        }

        let entry_to_leader = self.unionfind_get_impl(entry.r.id, map);
        let new = self.prove_transitivity(entry, entry_to_leader);

        map[i.0] = new.clone();
        new
    }

    pub fn unionfind_set(&self, i: Id, proof: ProvenEq) {
        let mut lock = self.unionfind.lock().unwrap();
        if lock.len() == i.0 {
            lock.push(proof);
        } else {
            lock[i.0] = proof;
        }
    }

    pub fn unionfind_get_proof(&self, i: Id) -> ProvenEq {
        let mut map = self.unionfind.lock().unwrap();
        self.unionfind_get_impl(i, &mut *map)
    }

    pub fn unionfind_get(&self, i: Id) -> AppliedId {
        self.unionfind_get_proof(i).r.clone()
    }

    pub fn unionfind_iter(&self) -> impl Iterator<Item=(Id, AppliedId)> {
        let mut map = self.unionfind.lock().unwrap();
        let mut out = Vec::new();

        for x in (0..map.len()).map(Id) {
            let y = self.unionfind_get_impl(x, &mut *map).r.clone();
            out.push((x, y));
        }

        out.into_iter()
    }

    pub fn unionfind_len(&self) -> usize {
        self.unionfind.lock().unwrap().len()
    }

    pub fn find_enode(&self, enode: &L) -> L {
        enode.map_applied_ids(|x| self.find_applied_id(&x))
    }

    // normalize i.id
    //
    // Example 1:
    // 'find(c1(s10, s11)) = c2(s11, s10)', where 'c1(s0, s1) -> c2(s1, s0)' in unionfind.
    //
    // Example 2:
    // 'find(c1(s3, s7, s8)) = c2(s8, s7)', where 'c1(s0, s1, s2) -> c2(s2, s1)' in unionfind,
    pub fn find_applied_id(&self, i: &AppliedId) -> AppliedId {
        let a = self.unionfind_get(i.id);

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
        self.unionfind_get(i).id
    }
}
