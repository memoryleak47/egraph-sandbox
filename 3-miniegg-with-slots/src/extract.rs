use crate::*;

use std::marker::PhantomData;

pub trait CostFn<L: Language> {
    fn cost<C>(enode: &L, costs: C) -> u64 where C: Fn(Id) -> u64;
}

pub struct AstSize<L: Language>(PhantomData<L>);

impl<L: Language> CostFn<L> for AstSize<L> {
    fn cost<C>(enode: &L, costs: C) -> u64 where C: Fn(Id) -> u64 {
        let mut s: u64 = 1;
        for x in enode.applied_id_occurences() {
            s = s.saturating_add(costs(x.id));
        }
        s
    }
}

fn rec_cost<L: Language, CF: CostFn<L>>(re: &RecExpr<L>) -> u64 {
    let mut costs = Vec::new();
    for x in &re.node_dag {
        let c = CF::cost(x, |i| costs[i.0]);
        costs.push(c);
    }
    costs.pop().unwrap()
}

pub fn ast_size_extract<L: Language>(i: Id, eg: &EGraph<L>) -> RecExpr<L> {
    extract::<L, AstSize<L>>(i, eg)
}

// our cost function is RecExpr::node_dag.len(), and we build every RecExpr s.t. each element of the node DAG is used exactly once.
// This is hence equivalent to AST size.
// `i` is not allowed to have free variables, hence prefer `Id` over `AppliedId`.
pub fn extract<L: Language, CF: CostFn<L>>(i: Id, eg: &EGraph<L>) -> RecExpr<L> {
    let i = eg.find_id(i);

    // this is a terribly slow algorithm.

    // maps eclass id to their optimal RecExpr.
    let mut map: HashMap<Id, RecExpr<L>> = HashMap::default();

    for _ in 0..eg.ids().len() {
        for id in eg.ids() {
            for n in eg.enodes(id) {
                if let Some(re) = extract_step(n, &map) {
                    // ENodes can have redundant nodes, hence it's "superset" instead of "equality".
                    assert!(re.node_dag.last().unwrap().slots().is_superset(&eg.slots(id)));

                    let new_cost = rec_cost::<L, CF>(&re);
                    let current_cost = map.get(&id).map(|x| rec_cost::<L, CF>(&x)).unwrap_or(u64::MAX);
                    if new_cost < current_cost {
                        map.insert(id, re);
                    }
                }
            }
        }
    }

    map.remove(&i).unwrap()
}

fn extract_step<L: Language>(enode: L, map: &HashMap<Id, RecExpr<L>>) -> Option<RecExpr<L>> {
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
