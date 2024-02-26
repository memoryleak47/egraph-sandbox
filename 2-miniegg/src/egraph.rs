use crate::*;

#[derive(Clone)]
struct EClass {
    // The set of equivalent ENodes that make up this eclass.
    nodes: Vec<ENode>, // de-duplicated and sorted.
}

// Invariants:
// 1. Each ENode comes up in at most one EClass::nodes. The reverse direction is then also stored in the hashcons.
// 2. Every Id - used anywhere within the EGraph - has to satisfy is_active()
pub struct EGraph {
    hashcons: HashMap<ENode, Id>,
    unionfind: HashMap<Id, Id>, // normalizes the eclass. The |x| unionfind[x] mapping is idempotent.
    classes: HashMap<Id, EClass>, // only ids with unionfind[x] = x are contained.
}

impl EGraph {
    pub fn new() -> Self {
        EGraph {
            hashcons: Default::default(),
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
        insert_node(&mut self.classes.get_mut(&i).unwrap().nodes, enode.clone());

        self.hashcons.insert(enode, i);

        i
    }

    pub fn lookup(&self, enode: &ENode) -> Option<Id> {
        let enode = self.normalize_enode(enode.clone());
        self.hashcons.get(&enode).cloned()
    }

    pub fn find(&self, i: Id) -> Id {
        self.unionfind[&i]
    }

    pub fn enodes(&self, i: Id) -> &[ENode] {
        let i = self.find(i);
        &self.classes[&i].nodes
    }

    fn enodes_mut(&mut self, i: Id) -> &mut Vec<ENode> {
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

        // move the nodes from r to l.
        for n in r_class.nodes {
            let old = self.hashcons.insert(n.clone(), l);
            assert_eq!(old, Some(r));

            self.enodes_mut(l).push(n);
        }

        // This search might be optimized by storing the "usages" (aka parents) of an eclass.
        let relevant_nodes: Vec<ENode> = self.hashcons.keys()
                                                  .cloned()
                                                  .filter(|n| &self.normalize_enode(n.clone()) != n)
                                                  .collect();

        let mut future_unions = Vec::new();
        for n in relevant_nodes {
            let nn = self.normalize_enode(n.clone());
            let c = self.hashcons.remove(&n).unwrap();
            remove_node(self.enodes_mut(c), &n);

            if let Some(c2) = self.hashcons.get(&nn) {
                future_unions.push((c, *c2));
            } else {
                self.hashcons.insert(nn.clone(), c);
                insert_node(self.enodes_mut(c), nn);
            }
        }

        for (a, b) in future_unions {
            self.union(a, b);
        }
    }

    pub fn alloc_eclass(&mut self) -> Id {
        let i = Id(self.unionfind.len());
        let eclass = EClass {
            nodes: vec![],
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

    // checks all invariants.
    pub fn assert_inv(&self) {
        let n = self.unionfind.len();
        for x in (0..n).map(Id) {
            let norm = self.unionfind[&x];
            assert_eq!(norm, self.unionfind[&norm]);
        }

        let mut counter = 0;
        for (&i, c) in self.classes.iter() {
            assert!(self.is_active(i));
            for node in &c.nodes {
                assert_eq!(self.hashcons[&node], i);
                counter += 1;
            }
        }

        assert_eq!(counter, self.hashcons.len());
    }
}

fn insert_node(nodes: &mut Vec<ENode>, enode: ENode) {
    // can be optimized using binary search.
    nodes.push(enode);
    nodes.sort();
    nodes.dedup();
}

fn remove_node(nodes: &mut Vec<ENode>, enode: &ENode) {
    // can be optimized using binary search.
    nodes.sort();
    nodes.dedup();
    let Ok(i) = nodes.binary_search(enode) else { panic!() };
    nodes.remove(i);
}
