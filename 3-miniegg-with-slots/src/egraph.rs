use crate::*;

#[derive(Clone)]
struct EClass {
    // The set of equivalent ENodes that make up this eclass.
    nodes: HashSet<ENode>,
}

// Invariants:
// 1. Each ENode comes up in at most one EClass::nodes.
// 2. Every Id - used anywhere within the EGraph - has to satisfy is_active()
pub struct EGraph {
    unionfind: HashMap<Id, Id>, // normalizes the eclass. The |x| unionfind[x] mapping is idempotent.
    classes: HashMap<Id, EClass>, // only ids with unionfind[x] = x are contained.
}

impl EGraph {
    pub fn new() -> Self {
        EGraph {
            unionfind: Default::default(),
            classes: Default::default(),
        }
    }

    pub fn add_expr(&mut self, re: RecExpr) -> Id {
        // vec maps Ids in `re` to Ids in the egraph.
        let mut vec = Vec::new();

        for x in re.node_dag {
            let x = x.map_ids(|i| vec[i.0]);
            vec.push(self.add(x));
        }
        *vec.last().unwrap()
    }

    fn normalize_enode(&self, enode: ENode) -> ENode {
        enode.map_ids(|x| self.find(x))
    }

    pub fn add(&mut self, enode: ENode) -> Id {
        let enode = self.normalize_enode(enode);

        if let Some(x) = self.lookup(&enode) {
            return x;
        }

        let i = self.alloc_eclass();
        self.classes.get_mut(&i).unwrap().nodes.insert(enode.clone());

        i
    }

    pub fn lookup(&self, enode: &ENode) -> Option<Id> {
        let enode = self.normalize_enode(enode.clone());

        for (&i, c) in self.classes.iter() {
            if c.nodes.contains(&enode) {
                return Some(i);
            }
        }

        None
    }

    pub fn find(&self, i: Id) -> Id {
        self.unionfind[&i]
    }

    pub fn enodes(&self, i: Id) -> &HashSet<ENode> {
        let i = self.find(i);
        &self.classes[&i].nodes
    }

    fn enodes_mut(&mut self, i: Id) -> &mut HashSet<ENode> {
        &mut self.eclass_mut(i).nodes
    }

    fn eclass_mut(&mut self, i: Id) -> &mut EClass {
        let i = self.find(i);
        self.classes.get_mut(&i).unwrap()
    }

    pub fn union(&mut self, l: Id, r: Id) {
        let l = self.find(l);
        let r = self.find(r);
        if l == r { return; }

        // add r -> l edge.
        self.unionfind.insert(r, l);

        // fix nested unionfind pointers.
        for i in (0..self.unionfind.len()).map(Id) { // might not be very fast this way..
            if self.unionfind[&i] == r {
                self.unionfind.insert(i, l);
            }
        }

        let r_class: EClass = self.classes.remove(&r).unwrap();

        self.enodes_mut(l).extend(r_class.nodes);

        // This search might be optimized by storing the "usages" (aka parents) of an eclass.
        let relevant_nodes: Vec<(Id, ENode)> = self.enode_iter()
                                                  .filter(|(_, n)| &self.normalize_enode(n.clone()) != n)
                                                  .collect();

        let mut future_unions = Vec::new();
        for (c, n) in relevant_nodes {
            let nn = self.normalize_enode(n.clone());
            self.enodes_mut(c).remove(&n);

            if let Some(c2) = self.lookup(&nn) {
                future_unions.push((c, c2));
            } else {
                self.enodes_mut(c).insert(nn);
            }
        }

        for (a, b) in future_unions {
            self.union(a, b);
        }
    }

    pub fn alloc_eclass(&mut self) -> Id {
        let i = Id(self.unionfind.len());
        let eclass = EClass {
            nodes: HashSet::new(),
        };

        self.classes.insert(i, eclass);
        self.unionfind.insert(i, i);

        i
    }

    pub fn is_active(&self, i: Id) -> bool {
        self.find(i) == i
    }

    pub fn ids(&self) -> Vec<Id> {
        self.classes.keys()
                    .copied()
                    .collect()
    }

    fn enode_iter(&self) -> impl Iterator<Item=(Id, ENode)> + '_ {
        self.classes.iter()
                    .map(|(i, c)| c.nodes.iter().map(|a| (*i, a.clone())))
                    .flatten()
    }
}
