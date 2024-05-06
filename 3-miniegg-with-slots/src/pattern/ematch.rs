use crate::*;

type Subst = HashMap<String, AppliedId>;

pub fn ematch<L: Language>(eg: &EGraph<L>, pattern: &Pattern<L>) -> Vec<Subst> {
    todo!()
}

struct Traversal<L: Language> {
    id: Id,
    node: Option<TraversalNode<L>>,
}

struct TraversalNode<L: Language> {
    shape: L,
    children: Vec<Traversal<L>>,
}
