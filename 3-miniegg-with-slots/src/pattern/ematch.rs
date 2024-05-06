use crate::*;

type Subst = HashMap<String, AppliedId>;

pub fn ematch<L: Language>(eg: &EGraph<L>, pattern: &Pattern<L>) -> Vec<Subst> {
    todo!()
}
