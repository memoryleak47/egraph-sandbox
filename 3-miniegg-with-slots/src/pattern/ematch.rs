use crate::*;

#[derive(Clone, PartialEq, Eq)]
enum SlotResult {
    Unmatched,
    MatchedTo(Slot),
    MatchedButLost,
}

type SlotResultMap = HashMap<Slot, SlotResult>;
type Subst = HashMap<String, AppliedId>;

struct Match {
    id: AppliedId, // this needs to be AppliedId, as your pattern might have free slots, like the pattern "(var s4)".
    subst: Subst,
}

// For each returned m: Match, we have pattern[m.subst] is represented by m.id
pub fn ematch<L: Language>(eg: &EGraph<L>, pattern: &Pattern<L>) -> Vec<Match> {
    let mut out = Vec::new();
    for id in eg.ids() {
        out.extend(ematch_impl(eg, pattern, id, SlotResultMap::default(), HashMap::default()));
    }
    out
}

// For each returned m: Match, we have
// - pattern[m.subst] is represented by m.id
// - m.subst is an extension of partial_subst
// - m.id.id == id
//
// srm: partially maps pattern-slotnames to slots(id).
fn ematch_impl<L: Language>(eg: &EGraph<L>, pattern: &Pattern<L>, id: Id, srm: SlotResultMap, partial_subst: Subst) -> Vec<Match> {
    match &pattern.node {
        ENodeOrVar::Var(s) => todo!(),
        ENodeOrVar::ENode(n1) => {
            let mut matches = Vec::new();
            'outer: for n2 in eg.enodes(id) {
                let mut srm = srm.clone();
                if let Some(smap) = superficial_match2(n1, &n2) {
                    for (x, y) in smap.iter() {
                        if !try_insert_compatible(x, SlotResult::MatchedTo(y), &mut srm) {
                            continue 'outer;
                        }
                    }
                }
                let sub_patterns = pattern.children.iter().cloned();
                let sub_app_ids = n2.applied_id_occurences().into_iter();

                let mut matches_local = Vec::new();
                for (pat, app_id) in sub_patterns.zip(sub_app_ids) {
                    let srm = todo!(); // rename the slots in the srm, according to app_id.
                    for a in ematch_impl(eg, &pat, app_id.id, srm, partial_subst) {
                        let a = todo!(); // undo the renaming from before. We should also affect *our* srm at this point.
                        matches_local.push(a);
                    }
                }
                matches.extend(matches_local);
            }
            matches
        },
    }
}

fn superficial_match<L: Language>(a: &L, b: &L) -> bool {
    return mk_superficial(a) == mk_superficial(b);

    fn mk_superficial<L: Language>(a: &L) -> L {
        let mut a = a.clone();
        for x in a.applied_id_occurences_mut() {
            *x = AppliedId::new(Id(0), SlotMap::new());
        }
        for x in a.all_slot_occurences_mut() {
            *x = Slot(0);
        }

        a
    }
}

// returns None, if they don't match.
// returne Some(smap) if they agree, just with renaming smap in the slots.
// Ignores all AppliedIds and their slots.
fn superficial_match2<L: Language>(a: &L, b: &L) -> Option<SlotMap> {
    let a = remove_applied_ids(a);
    let b = remove_applied_ids(b);

    let mut smap = SlotMap::new();
    let a_slots = a.all_slot_occurences();
    let b_slots = b.all_slot_occurences();
    if a_slots.len() != b_slots.len() { return None; }
    for i in 0..a_slots.len() {
        let x = a_slots[i];
        let y = b_slots[i];
        if let Some(z) = smap.get(x) {
            if y != z { return None; }
        }
        smap.insert(x, y);
    }

    let a = remove_slot_names(&a);
    let b = remove_slot_names(&b);
    return if a == b { Some(smap) } else { None };

    fn remove_applied_ids<L: Language>(a: &L) -> L {
        let mut a = a.clone();
        for x in a.applied_id_occurences_mut() {
            *x = AppliedId::new(Id(0), SlotMap::new());
        }
        a
    }

    fn remove_slot_names<L: Language>(a: &L) -> L {
        let mut a = a.clone();
        for x in a.all_slot_occurences_mut() {
            *x = Slot(0);
        }
        a
    }
}

// returns pattern[subst].
// replaces all occurences of ?-variables in `pattern` with the corresponding AppliedId given by `subst`.
// Then does something like EGraph::add_expr on the result.
//
// Is equivalent to replacing all ?-variables x, with a term extracted for the AppliedId subst[x], translating the Pattern<L> to RecExpr2<L>,
// followed by EGraph::add_expr.
fn pattern_subst<L: Language>(pattern: &Pattern<L>, subst: &Subst, eg: &mut EGraph<L>) -> AppliedId {
    todo!()
}

fn try_insert_compatible<K: Eq + Hash, V: Eq>(k: K, v: V, map: &mut HashMap<K, V>) -> bool {
    if let Some(v2) = map.get(&k) {
        return &v == v2;
    }
    map.insert(k, v);
    true
}
