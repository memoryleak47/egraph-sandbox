use crate::*;

mod cost;
pub use cost::*;

mod with_ord;
pub use with_ord::*;

use std::collections::BinaryHeap;

pub fn ast_size_extract<L: Language>(i: AppliedId, eg: &EGraph<L>) -> RecExpr<L> {
    extract::<L, AstSize<L>>(i, eg)
}

// `i` is not allowed to have free variables, hence prefer `Id` over `AppliedId`.
pub fn extract<L: Language, CF: CostFunction<L>>(i: AppliedId, eg: &EGraph<L>) -> RecExpr<L> {
    let i = eg.find_applied_id(&i);

    // maps eclass id to their optimal RecExpr.
    let mut map: HashMap<Id, WithOrdRev<L, CF::Cost>> = HashMap::default();
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
        map.insert(i.id, WithOrdRev(enode, c));

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

    extract_final(i, &|i| &map[&i].0)
}

fn extract_final<'a, L: Language + 'a>(i: AppliedId, map: &'a impl Fn(Id) -> &'a L) -> RecExpr<L> {
    let mut children = Vec::new();

    // do I need to refresh some slots here?
    let l = map(i.id).apply_slotmap(&i.m);
    for child in l.applied_id_occurences() {
        let n = extract_final(child, map);
        children.push(n);
    }
    RecExpr {
        node: l,
        children,
    }
}
