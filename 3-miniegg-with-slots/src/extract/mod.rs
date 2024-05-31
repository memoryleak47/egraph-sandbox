use crate::*;

mod cost;
pub use cost::*;

mod with_ord;
pub use with_ord::*;

use std::collections::BinaryHeap;

pub fn ast_size_extract<L: Language>(i: AppliedId, eg: &EGraph<L>) -> RecExpr<L> {
    extract::<L, AstSize<L>>(i, eg)
}

// Does this work with non-trivial AppliedIds?
pub fn extract<L: Language, CF: CostFunction<L>>(i: AppliedId, eg: &EGraph<L>) -> RecExpr<L> {
    let i = eg.find_applied_id(&i);

    // maps eclass id to their optimal RecExpr.
    // TODO the map doesn't need RecExpr<L>. Just storing L would be enough.
    let mut map: HashMap<Id, WithOrdRev<RecExpr<L>, CF::Cost>> = HashMap::default();
    let mut queue: BinaryHeap<WithOrdRev<L, CF::Cost>> = BinaryHeap::new();

    for id in eg.ids() {
        for x in eg.enodes(id) {
            if x.applied_id_occurences().is_empty() {
                let c = CF::cost(&x, |_| panic!());
                queue.push(WithOrdRev(x, c));
            }
        }
    }

    while let Some(WithOrdRev(enode, c)) = queue.pop() {
        let i = eg.lookup(&enode).unwrap();
        if map.contains_key(&i.id) {
            continue;
        }
        let map_fn = |i| map[&i].0.clone();
        let re = extract_step(enode.clone(), &map_fn);
        map.insert(i.id, WithOrdRev(re, c));

        for x in eg.usages(i.id).clone() {
            if x.applied_id_occurences().iter().all(|i| map.contains_key(&i.id)) {
                if eg.lookup(&x).map(|i| map.contains_key(&i.id)).unwrap_or(false) {
                    continue;
                }
                let c = CF::cost(&x, |i| map[&i].1.clone());
                queue.push(WithOrdRev(x, c));
            }
        }
    }

    // TODO apply_slotmap correct?
    let mut re = map.remove(&i.id).unwrap().0;
    let rf: &mut L = re.node_dag.last_mut().unwrap();
    *rf = rf.apply_slotmap(&i.m);
    re
}

fn extract_step<L: Language>(enode: L, map: &impl Fn(Id) -> RecExpr<L>) -> RecExpr<L> {
    let mut c = enode.clone();
    let mut re = RecExpr::empty();
    for x in c.applied_id_occurences_mut() {
        let re2 = map(x.id);
        re.extend(re2);
        x.id = re.head_id();
    }
    re.push(c);

    re
}
