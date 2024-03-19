use crate::*;

// returns an eclass containing b[x := t]
// out has slots (slots(b) - {x}) cup slots(t).
// I presume that slots(t) is allowed to contain x.
pub fn subst(b: AppliedId, x: Slot, t: AppliedId, eg: &mut EGraph) -> AppliedId {
    subst_impl(b, x, t, eg, &mut Default::default())
}

fn subst_impl(b: AppliedId, x: Slot, t: AppliedId, eg: &mut EGraph, map: &mut Map) -> AppliedId {
    if !b.slots().contains(&x) {
        return b;
    }

    let new_b = match map_lookup(&b, x, &t, eg, map) {
        Ok(map_b) => return map_b,
        Err(new_b) => new_b,
    };

    for enode in eg.enodes_applied(&b) {
        let app_id = enode_subst(enode, &b, x, &t, eg, map);
        eg.union(new_b.clone(), app_id);
    }

    new_b
}

// `enode` is an enode from the eclass `b`.
// we return an eclass containing `enode[x := t]`
fn enode_subst(enode: ENode, b: &AppliedId, x: Slot, t: &AppliedId, eg: &mut EGraph, map: &mut Map) -> AppliedId {
    todo!()
    /*
    match enode {
        // (lam x2 b2)[x := t] --> (lam x2 b2[x := t]), if x != x2.
        ENode::Lam(x2, b2) => {
            assert!(x2 != x);

            let b2 = subst_impl(b2, x, t, eg, map);
            Some(eg.add(ENode::Lam(x2, b2)))
        }

        // (app l r)[x := t] --> (app l[x := t] r[x := t])
        ENode::App(l, r) => {
            let l = subst_impl(l, x, t, eg, map);
            let r = subst_impl(r, x, t, eg, map);
            Some(eg.add(ENode::App(l, r)))
        },

        // x2[x := t] --> t, if x = x2.
        ENode::Var(x2) if x == x2 => Some(t),

        // x2[x := t] --> x2, if x != x2.
        ENode::Var(_) => Some(b),
    }
    */
}

/////////////// Map impl ///////////////

#[derive(PartialEq, Eq, Hash, Debug)]
struct Key {
    b: Id,
    x: Slot,

    // (w1, w2) in bt_relation, iff w1 in slots(b) & w2 in slots(t), and
    // if w1 and w2 correspond to the same slot in this subst instance.
    // Note that b and t are Ids in this context.
    bt_relation: SlotMap,
}

struct Value {
    // the returned Id
    out_id: Id,

    // b: Id
    // maps (slots(b) - {x}) -> slots(out_id)
    b_map: SlotMap,

    // t: Id
    // maps slots(t) -> slots(out_id)
    t_map: SlotMap,

    // b_map and t_map need to be consistent with the bt_relation:
    // if (w1, w2) in bt_relation, then b_map[w1] == t_map[w2].
}

type Map = HashMap<Key, Value>;

// Ok(app_id) means that it was already in the map, and nothing needs to be done.
// Err(app_id) means that it was not yet in the map, but a new entry was added for it. Go and union stuff to it!
// Either way slots(app_id) == (slots(b) - {x}) & slots(t).
fn map_lookup(b: &AppliedId, x: Slot, t: &AppliedId, eg: &mut EGraph, map: &mut Map) -> Result<AppliedId, AppliedId> {
    todo!()
    /*
        // the else case:
        let max_slots = &(&b.slots() - &HashSet::from([x])) & &t.slots();
        let fresh = SlotMap::bijection_from_fresh_to(&max_slots);
        let slots = fresh.keys();

        let new_b = eg.alloc_eclass(&slots);
        map.insert(b, new_b);
    */
}
