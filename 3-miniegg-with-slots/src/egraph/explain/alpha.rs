use crate::*;

pub fn alpha_eq<L: Language>(a: &RecExpr<L>, b: &RecExpr<L>) -> bool {
    alpha_eq_impl(a, b, Default::default())
}

// we assume that all slots come up either free, or bound but not both inside of a single term.
// `map` maps the *bound* slot names from a to b.
fn alpha_eq_impl<L: Language>(a: &RecExpr<L>, b: &RecExpr<L>, map: SlotMap) -> bool {
    let mut map = map;

    // weak shape check.
    if a.node.weak_shape().0 != b.node.weak_shape().0 {
        return false;
    }

    // private slot introduction.
    let sa = a.node.private_slot_occurences().into_iter();
    let sb = b.node.private_slot_occurences().into_iter();

    for (x, y) in sa.zip(sb) {
        if map.keys().contains(&x) { return false; }
        if map.values().contains(&y) { return false; }
        map.insert(x, y);
    }

    // general slot check.
    let sa = a.node.all_slot_occurences().into_iter();
    let sb = b.node.all_slot_occurences().into_iter();

    for (x, y) in sa.zip(sb) {
        if map.keys().contains(&x) || map.values().contains(&y) {
            // check bound slot equality.
            if map.get(x) != Some(y) { return false; }
        } else {
            // check unbound slot equality.
            if x != y { return false; }
        }
    }

    // recursion check.
    for (l, r) in a.children.iter().zip(b.children.iter()) {
        if !alpha_eq_impl(l, r, map.clone()) { return false; }
    }

    true
}
