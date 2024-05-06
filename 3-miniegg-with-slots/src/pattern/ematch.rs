use crate::*;

pub type Subst = HashMap<String, AppliedId>;

pub fn ematch<L: Language>(eg: &EGraph<L>, pattern: &Pattern<L>) -> Vec<Subst> {
    let mut out = Vec::new();
    for i in eg.ids() {
        // invariant: each x in worklist satisfies x.compatible(pattern)
        let mut worklist = vec![Traversal::new(i)];
        while let Some(x) = worklist.pop() {
            if let Some(xs) = x.branch(pattern, eg) {
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

#[derive(Clone)]
struct Traversal<L: Language> {
    id: Id,
    node: Option<TraversalNode<L>>,
}

#[derive(Clone)]
struct TraversalNode<L: Language> {
    l: L,
    children: Vec<Traversal<L>>,
}

impl<L: Language> Traversal<L> {
    pub fn new(id: Id) -> Self {
        Traversal {
            id,
            node: None,
        }
    }

    // If the Traversal already covers the whole pattern, we return None.
    // Otherwise, we extend the Traversal at some point and return all possible e-node extensions for that spot.
    pub fn branch(&self, pattern: &Pattern<L>, eg: &EGraph<L>) -> Option<Vec<Self>> {
        match (&self.node, &pattern.node) {
            // Here we can extend the Traversal:
            (None, ENodeOrVar::ENode(n)) => {
                let mut out = Vec::new();
                for l in eg.enodes(self.id) {
                    let tr_node = TraversalNode {
                        l: l.clone(),
                        children: l.applied_id_occurences().into_iter().map(|x| Traversal::new(x.id)).collect(),
                    };
                    let tr = Traversal {
                        id: self.id,
                        node: Some(tr_node),
                    };
                    out.push(tr);
                }
                Some(out)
            },
            (Some(x), ENodeOrVar::ENode(n)) => {
                assert_eq!(x.children.len(), pattern.children.len());
                for i in 0..x.children.len() {
                    let subtrav = &x.children[i];
                    let subpat = &pattern.children[i];
                    if let Some(subs) = subtrav.branch(subpat, eg) {
                        let mut out = Vec::new();
                        for sub in subs {
                            let mut option = self.clone();
                            let rf = option.node.as_mut().unwrap();
                            rf.children[i] = sub;
                            out.push(option);
                        }
                        return Some(out);
                    }
                }
                None
            },
            (None, ENodeOrVar::Var(_)) => None,
            (Some(_), ENodeOrVar::Var(_)) => panic!(),
        }
    }

    pub fn compatible(&self, pattern: &Pattern<L>) -> bool {
        todo!()
    }

    pub fn to_subst(&self, pattern: &Pattern<L>) -> Subst {
        todo!()
    }
}
