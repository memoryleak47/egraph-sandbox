use crate::*;

// our cost function is RecExpr::node_dag.len(), and we build every RecExpr s.t. each element of the node DAG is used exactly once.
// This is hence equivalent to AST size.
// `i` is not allowed to have free variables, hence prefer `Id` over `AppliedId`.
pub fn extract(i: Id, eg: &EGraph) -> RecExpr {
    let i = eg.normalize_id_by_unionfind(i);

    // this is a terribly slow algorithm.

    // maps eclass id to their optimal RecExpr.
    let mut map: HashMap<Id, RecExpr> = HashMap::new();

    for _ in 0..eg.ids().len() {
        for id in eg.ids() {
            for n in eg.enodes(id) {
                let db = |a| db_impl(a, &map);
                if let Some(re) = extract_step(n, &db) {
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

fn extract_step(enode: ENode, db: &impl Fn(AppliedId) -> Option<RecExpr>) -> Option<RecExpr> {
    match enode {
        ENode::Var(x) => {
            let re = RecExpr { node_dag: vec![ENode::Var(x)] };

            Some(re)
        },
        ENode::Lam(x_old, b_old) => {
            // rename x_old to a fresh name. Otherwise it might collide with other s0 lambdas later on.
            let x = Slot::fresh();
            let mut m = SlotMap::identity(&b_old.slots());
            m.insert(x_old, x);
            let b = b_old.apply_slotmap(&m);


            let mut re = db(b)?;
            let last = Id(re.node_dag.len() - 1);
            let last_slots = re.node_dag.last().unwrap().slots(); // TODO correct?
            let last = AppliedId::new(last, SlotMap::identity(&last_slots));

            let enode = ENode::Lam(x, last);
            re.node_dag.push(enode);

            Some(re)
        },
        ENode::App(l, r) => {
            let l = db(l)?;
            let r = db(r)?;
            let last1 = Id(l.node_dag.len() - 1);
            let last1_slots = l.node_dag.last().unwrap().slots(); // TODO correct?
            let last1 = AppliedId::new(last1, SlotMap::identity(&last1_slots));

            let n = l.node_dag.len();
            let shift_n = |x: AppliedId| AppliedId::new(Id(x.id.0 + n), x.m);
            let r = r.node_dag.iter()
                              .map(|enode| enode.map_applied_ids(shift_n));
            let mut re = l;
            re.node_dag.extend(r);

            let last2 = Id(re.node_dag.len() - 1);
            let last2_slots = re.node_dag.last().unwrap().slots(); // TODO correct?
            let last2 = AppliedId::new(last2, SlotMap::identity(&last2_slots));

            let enode = ENode::App(last1, last2);
            re.node_dag.push(enode);

            Some(re)
        },
    }
}

// a simple lookup of `map[a]`, but wait! `a` is an AppliedId instead of a simple Id.
// Hence we need to do some renaming.
// if Some(re) = db_impl(a, ..), then re.last().slots() = a.slots()
fn db_impl(a: AppliedId, map: &HashMap<Id, RecExpr>) -> Option<RecExpr> {
    let mut re: RecExpr = map.get(&a.id)?.clone();
    let b: &mut ENode = re.node_dag.last_mut().unwrap();

    // a.slots() == A
    // re.slots() == b.slots() == eg.slots(a.id) == B
    // a.m :: B -> A

    *b = b.apply_slotmap(&a.m);

    assert_eq!(b.slots(), a.slots());

    Some(re)
}
