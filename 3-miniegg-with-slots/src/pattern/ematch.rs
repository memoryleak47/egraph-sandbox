use crate::*;

type Subst = HashMap<String, AppliedId>;

pub fn ematch<L: Language>(eg: &EGraph<L>, pattern: &Pattern<L>) -> Vec<Subst> {
    let mut out = Vec::new();
    for i in eg.ids() {
        // invariant: each x in worklist satisfies x.compatible(pattern)
        let mut worklist = vec![Traversal::new(i)];
        while let Some(x) = worklist.pop() {
            if let Some(xs) = x.branch(pattern) {
                for y in xs {
                    if y.compatible(pattern) {
                        worklist.push(y);
                    }
                }
            } else {
                out.push(x.to_subst(pattern));
            }
        }
    }
    out
}

struct Traversal<L: Language> {
    id: Id,
    node: Option<TraversalNode<L>>,
}

struct TraversalNode<L: Language> {
    shape: L,
    children: Vec<Traversal<L>>,
}

impl<L: Language> Traversal<L> {
    pub fn new(id: Id) -> Self {
        Traversal {
            id,
            node: None,
        }
    }

    pub fn branch(&self, pattern: &Pattern<L>) -> Option<Vec<Self>> {
        todo!()
    }

    pub fn compatible(&self, pattern: &Pattern<L>) -> bool {
        todo!()
    }

    pub fn to_subst(&self, pattern: &Pattern<L>) -> Subst {
        todo!()
    }
}
