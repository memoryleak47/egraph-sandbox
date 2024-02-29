use crate::*;

// our cost function is RecExpr::node_dag.len(), and we build every RecExpr s.t. each element of the node DAG is used exactly once.
// This is hence equivalent to AST size.
// `i` is not allowed to have free variables, hence prefer `Id` over `AppliedId`.
pub fn extract(i: Id, eg: &EGraph) -> RecExpr {
    let i = eg.find_id(i);

    // this is a terribly slow algorithm.

    // maps eclass id to their optimal RecExpr.
    let mut map: HashMap<Id, RecExpr> = HashMap::new();

    for _ in 0..eg.ids().len() {
        for id in eg.ids() {
            for n in eg.enodes(id) {
                let db = |a| db_impl(a, &map, eg);
                if let Some(re) = extract_step(n, &db, eg) {
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

fn extract_step(n: ENode, db: &impl Fn(AppliedId) -> Option<RecExpr>, eg: &EGraph) -> Option<RecExpr> {
    /*
    match n {
        ENode::Var(x) => {
            RecExpr { node_dag: vec![ENode::Var(x.clone())] }
        },
        ENode::Lam(x, b) => {
            let Some(mut re) = map.get(b).cloned() else { continue };
            let last = Id(re.node_dag.len()-1);
            let enode = ENode::Lam(x.clone(), last);
            re.node_dag.push(enode);
            re
        },
        ENode::App(l, r) => {
            let Some(l) = map.get(l).cloned() else { continue };
            let last1 = Id(l.node_dag.len() - 1);

            let n = l.node_dag.len();
            let f = |Id(x)| Id(x + n);
            let Some(r) = map.get(r) else { continue };
            let r = r.node_dag.iter()
                              .map(|enode| enode.clone().map_ids(f));
            let mut re = l;
            re.node_dag.extend(r);

            let last2 = Id(re.node_dag.len() - 1);

            let enode = ENode::App(last1, last2);
            re.node_dag.push(enode);

            re
        },
    }
    */
    todo!()
}

// a simple lookup of `map[a]`, but wait! `a` is an AppliedId instead of a simple Id.
// Hence we need to do some renaming.
fn db_impl(a: AppliedId, map: &HashMap<Id, RecExpr>, eg: &EGraph) -> Option<RecExpr> {
    todo!()
}
