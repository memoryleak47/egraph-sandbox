use crate::*;

type Subst = HashMap<String, AppliedId>;

struct Match {
    id: AppliedId, // this needs to be AppliedId, as your pattern might have free slots, like the pattern "(var s4)".
    subst: Subst,
}

pub fn ematch<L: Language>(eg: &EGraph<L>, pattern: &Pattern<L>) -> Vec<Match> {
    let mut out = Vec::new();
    for id in eg.ids() {
        out.extend(ematch_impl(eg, pattern, id, HashMap::default()));
    }
    out
}

fn ematch_impl<L: Language>(eg: &EGraph<L>, pattern: &Pattern<L>, id: Id, partial_subst: Subst) -> Vec<Match> {
    match &pattern.node {
        ENodeOrVar::Var(s) => {
            // TODO is this right?
            let mut subst = partial_subst.clone();
            if !subst.contains_key(&*s) {
                let app_id = AppliedId::new(id, todo!());
                subst.insert(s.clone(), app_id);
            }
            let mtch = Match { id: todo!(), subst };
            vec![mtch]
        },
        ENodeOrVar::ENode(n1) => {
            // TODO is this right?
            let mut matches = Vec::new();
            for n2 in eg.enodes(id) {
                if superficial_match(n1, &n2) {
                    let mut subst = partial_subst.clone();
                    let mtch = todo!();
                    matches.push(mtch);
                }
            }
            matches
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
