use crate::*;

use std::sync::Mutex;

#[derive(Default, Debug)]
pub(in crate::egraph) struct Unionfind {
    // "map: HashMap<Id, Cell<AppliedId>>" is probably the optimal single-threaded choice.
    //
    // "map: Vec<RwLock<AppliedId>>" might be similarly good, as we mostly read.
    // And only if get() notices a non-normalized entry, we need to lock mutably.
    //
    map: Mutex<Vec<AppliedId>>,
}

fn get_impl(i: Id, map: &mut [AppliedId]) -> AppliedId {
    let next = map[i.0].clone();

    if next.id == i {
        return next;
    }

    // repr.id is the final representant of i.
    let repr = get_impl(next.id, map);

    // next.m :: slots(next.id) -> slots(i)
    // repr.m :: slots(repr.id) -> slots(next.id)

    // out.m :: slots(repr.id) -> slots(i)
    let out = repr.apply_slotmap(&next.m);

    map[i.0] = out.clone();
    out
}

impl Unionfind {
    pub fn set(&self, i: Id, j: &AppliedId) {
        let mut lock = self.map.lock().unwrap();
        if lock.len() == i.0 {
            lock.push(j.clone());
        } else {
            lock[i.0] = j.clone();
        }
    }

    pub fn get(&self, i: Id) -> AppliedId {
        let mut map = self.map.lock().unwrap();
        get_impl(i, &mut *map)
    }

    pub fn iter(&self) -> impl Iterator<Item=(Id, AppliedId)> {
        let mut map = self.map.lock().unwrap();
        let mut out = Vec::new();

        for x in (0..map.len()).map(Id) {
            let y = get_impl(x, &mut *map);
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
        let a = &self.unionfind.get(i.id);

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
        self.unionfind.get(i).id
    }
}
