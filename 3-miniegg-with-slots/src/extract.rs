use crate::*;

// our cost function is RecExpr::node_dag.len(), and we build every RecExpr s.t. each element of the node DAG is used exactly once.
// This is hence equivalent to AST size.
// `i` is not allowed to have free variables, hence prefer `Id` over `AppliedId`.
pub fn extract(i: Id, eg: &EGraph<ENode>) -> RecExpr<ENode> {
    let i = eg.find_id(i);

    // this is a terribly slow algorithm.

    // maps eclass id to their optimal RecExpr.
    let mut map: HashMap<Id, RecExpr<ENode>> = HashMap::default();

    for _ in 0..eg.ids().len() {
        for id in eg.ids() {
            for n in eg.enodes(id) {
                if let Some(re) = extract_step(n, &map) {
                    // ENodes can have redundant nodes, hence it's "superset" instead of "equality".
                    assert!(re.node_dag.last().unwrap().slots().is_superset(&eg.slots(id)));

                    let new_cost = re.node_dag.len();
                    let current_cost = map.get(&id).map(|x| x.node_dag.len()).unwrap_or(usize::MAX);
                    if new_cost < current_cost {
                        map.insert(id, re);
                    }
                }
            }
        }
    }

    map.remove(&i).unwrap()
}

fn extract_step(enode: ENode, map: &HashMap<Id, RecExpr<ENode>>) -> Option<RecExpr<ENode>> {
    let mut c = enode.clone();
    let mut re = RecExpr::empty();
    for x in c.applied_id_occurences_mut() {
        let re2 = map.get(&x.id)?.clone();
        re.extend(re2);
        x.id = re.head_id();
    }
    re.push(c);

    Some(re)
}
