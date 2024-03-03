use crate::*;

#[derive(Clone, Debug)]
struct EClass {
    nodes: HashSet<ENode>, // The set of equivalent ENodes that make up this eclass.
    free_slot_count: usize, // free slots = [0..free_slot_count].
}

#[derive(Debug)]
pub struct EGraph {
    // an entry (l, r(sa, sb)) in unionfind corresponds to the equality l(s0, s1, s2) = r(sa, sb), where sa, sb in {s0, s1, s2}.
    unionfind: HashMap<Id, AppliedId>, // normalizes the eclass. is "idempotent".
    classes: HashMap<Id, EClass>, // only ids with unionfind[x].id = x are contained.
}

impl EGraph {
    pub fn new() -> Self {
        EGraph {
            unionfind: Default::default(),
            classes: Default::default(),
        }
    }
}
