use crate::*;

use std::sync::Mutex;

type Entry = (AppliedId, Arc<ProvenEq>);

#[derive(Default, Debug)]
pub(in crate::egraph) struct Unionfind {
    // "map: HashMap<Id, Cell<AppliedId>>" is probably the optimal single-threaded choice.
    //
    // "map: Vec<RwLock<AppliedId>>" might be similarly good, as we mostly read.
    // And only if get() notices a non-normalized entry, we need to lock mutably.
    //
    map: Mutex<Vec<(AppliedId, Arc<ProvenEq>)>>,
}

// We lazily semify the entries, only when we encounter them.
fn semify_entry<L: Language>(i: Id, entry: &mut Entry, eg: &EGraph<L>) {
    let app_id = &mut entry.0;
    if app_id.m.keys() != eg.slots(app_id.id) {
        // gets rid of redundant slots that we didn't yet put in the unionfind.
        *app_id = eg.semify_app_id(app_id.clone());
    }
    // TODO also update proof by transitivity with the shrink proof.
    // This would also be necessary when the left-hand side of the equation shrinks.
}

// TODO everything now also depends on eg. This is a mess. Let's clean this up.
fn get_impl<L: Language>(i: Id, map: &mut [Entry], eg: &EGraph<L>) -> (AppliedId, Arc<ProvenEq>) {
    let entry = &mut map[i.0];
    semify_entry(i, entry, eg);

    let mut next = entry.clone();

    if next.0.id == i {
        return next;
    }

    // repr.id is the final representant of i.
    let repr = get_impl(next.0.id, map, eg);

    // next.m :: slots(next.id) -> slots(i)
    // repr.m :: slots(repr.id) -> slots(next.id)

    // out.m :: slots(repr.id) -> slots(i)
    let out_app_id = repr.0.apply_slotmap(&next.0.m);
    let out = (out_app_id, repr.1);

    map[i.0] = out.clone();
    out
}

impl Unionfind {
    pub fn set(&self, i: Id, j: AppliedId, proof: Arc<ProvenEq>) {
        let mut lock = self.map.lock().unwrap();
        if lock.len() == i.0 {
            lock.push((j, proof));
        } else {
            lock[i.0] = (j, proof);
        }
    }

    pub fn get_proof<L: Language>(&self, i: Id, eg: &EGraph<L>) -> (AppliedId, Arc<ProvenEq>) {
        let mut map = self.map.lock().unwrap();
        get_impl(i, &mut *map, eg)
    }

    pub fn get<L: Language>(&self, i: Id, eg: &EGraph<L>) -> AppliedId {
        self.get_proof(i, eg).0
    }

    pub fn iter<L: Language>(&self, eg: &EGraph<L>) -> impl Iterator<Item=(Id, AppliedId)> {
        let mut map = self.map.lock().unwrap();
        let mut out = Vec::new();

        for x in (0..map.len()).map(Id) {
            let y = get_impl(x, &mut *map, eg).0;
            out.push((x, y));
        }

        out.into_iter()
    }

    pub fn len(&self) -> usize {
        self.map.lock().unwrap().len()
    }
}

impl<L: Language> EGraph<L> {
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
        let a = self.unionfind.get(i.id, self);

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
        self.unionfind.get(i, self).id
    }
}
