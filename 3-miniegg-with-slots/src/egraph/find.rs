use crate::*;

pub type Entry = (AppliedId, Arc<ProvenEq>);

impl<L: Language> EGraph<L> {
    // We lazily semify the entries, only when we encounter them.
    fn unionfind_semify_entry(&self, i: Id, entry: &mut Entry) {
        let app_id = &mut entry.0;
        if app_id.m.keys() != self.slots(app_id.id) {
            // gets rid of redundant slots that we didn't yet put in the unionfind.
            *app_id = self.semify_app_id(app_id.clone());
        }
        // TODO also update proof by transitivity with the shrink proof.
        // This would also be necessary when the left-hand side of the equation shrinks.
    }

    fn unionfind_get_impl(&self, i: Id, map: &mut [Entry]) -> (AppliedId, Arc<ProvenEq>) {
        let entry = &mut map[i.0];
        self.unionfind_semify_entry(i, entry);

        let mut next = entry.clone();

        if next.0.id == i {
            return next;
        }

        // repr.id is the final representant of i.
        let repr = self.unionfind_get_impl(next.0.id, map);

        // next.m :: slots(next.id) -> slots(i)
        // repr.m :: slots(repr.id) -> slots(next.id)

        // out.m :: slots(repr.id) -> slots(i)
        let out_app_id = repr.0.apply_slotmap(&next.0.m);
        let out = (out_app_id, repr.1);

        map[i.0] = out.clone();
        out
    }

    pub fn unionfind_set(&self, i: Id, j: AppliedId, proof: Arc<ProvenEq>) {
        let mut lock = self.unionfind.lock().unwrap();
        if lock.len() == i.0 {
            lock.push((j, proof));
        } else {
            lock[i.0] = (j, proof);
        }
    }

    pub fn unionfind_get_proof(&self, i: Id) -> (AppliedId, Arc<ProvenEq>) {
        let mut map = self.unionfind.lock().unwrap();
        self.unionfind_get_impl(i, &mut *map)
    }

    pub fn unionfind_get(&self, i: Id) -> AppliedId {
        self.unionfind_get_proof(i).0
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
