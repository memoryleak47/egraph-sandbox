use crate::*;

struct Match {
    id: Id,
    subst: HashMap<String, AppliedId>,
}

pub fn ematch<L: Language>(eg: &EGraph<L>, pattern: &Pattern<L>) -> Vec<Match> {
    let mut out = Vec::new();
    for id in eg.ids() {
        out.extend(ematch_impl(eg, pattern, id, HashMap::default()));
    }
    out
}

fn ematch_impl<L: Language>(eg: &EGraph<L>, pattern: &Pattern<L>, id: Id, partial_subst: HashMap<String, AppliedId>) -> Vec<Match> {
    match pattern.node {
        ENodeOrVar::Var(_) => {
            todo!()
        },
        ENodeOrVar::ENode(_) => {
            todo!()
        },
    }
}

fn superficial_match<L: Language>(a: &L, b: &L) -> bool {
    return mk_superficial(a) == mk_superficial(b);

    fn mk_superficial<L: Language>(a: &L) -> L {
        let mut a = a.clone();
        for x in a.applied_id_occurences_mut() {
            *x = AppliedId::new(Id(0), SlotMap::new());
        }
        for x in a.all_slot_occurences_mut() {
            *x = Slot(0);
        }

        a
    }
}
