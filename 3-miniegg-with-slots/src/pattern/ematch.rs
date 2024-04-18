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
