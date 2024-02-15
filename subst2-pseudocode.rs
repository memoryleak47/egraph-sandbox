use crate::*;

use std::collections::HashMap;

pub fn subst2() -> Rewrite<Term, ()> {
    rewrite!("beta-reduction2"; "(app (lam ?x ?b) ?t)" => { BetaReduction })
}

struct BetaReduction;

// returns b[x := t]
fn substitute(b: Id, x: Id, t: Id, eg: &mut EGraph<Term, ()>) -> Id {
    substitute_impl(b, x, t, eg, &mut HashMap::new())
}

// returns b[x := t].
// `map` cathes the b -> b[x := t] mapping.
fn substitute_impl(b: Id, x: Id, t: Id, eg: &mut EGraph<Term, ()>, map: &mut HashMap<Id, Id>) -> Id {
    if let Some(o) = map.get(b) {
        return o;
    }

    // new_b represents the e-class `b[x := t]`.
    let new_b = eg.allocate_empty_eclass();
    map.insert(b, new_b);

    for enode in eg[b].nodes.clone() {
        let i = enode_subst(enode, b, x, t, eg, map);
        eg.union(i, new_b);
    }

    new_b
}

// TODO this might be easier to follow, if we return another enode instead of returning an eclass.
fn enode_subst(enode: ENode, b: Id, x: Id, t: Id, eg: &mut EGraph<Term, ()>, map: &mut HashMap<Id, Id>) -> Id {
    match enode {
        // if we encounter `(lam x b2)`, we return `(lam x b2)`, i.e. we don't change anything.
        ENode::Abstraction([x2, _]) if eg.find(x2) == eg.find(x) => b,

        // if we encounter `(lam x2 b2)` (x2 != x), we return `(lam x2 b2[x := t])`.
        ENode::Abstraction([x2, b2]) => {
            let id = substitute_impl(b2, x, t, eg, map);
            eg.add(ENode::Abstraction([x2, id]))
        }

        // if we encounter `(app l r)`, we return `(app l[x := t] r[x := t])`.
        ENode::Application([l, r]) => {
            let l = substitute_impl(l, x, t, eg, map);
            let r = substitute_impl(r, x, t, eg, map);
            eg.add(ENode::Application([l, r]))
        },

        // if we encounter `x`, we return `t`.
        ENode::Symb(_) if eg.find(x) == eg.find(b) => t,

        // if we encounter `x2` (x != x2), we return `x2`.
        ENode::Symb(_) => b,

        // if we encounter a number, we return it unchanged.
        ENode::Num(_) => b,
    }
}

impl Applier<Term, ()> for BetaReduction {
    fn apply_one(&self, eg: &mut EGraph<Term, ()>, id: Id, subst: &Subst, _pat: Option<&PatternAst<Term>>, _rule_name: Symbol) -> Vec<Id> {
        let b: Var = "?b".parse().unwrap();
        let x: Var = "?x".parse().unwrap();
        let t: Var = "?t".parse().unwrap();

        let b: Id = subst[b];
        let x: Id = subst[x];
        let t: Id = subst[t];

        let new = substitute(b, x, t, eg);
        eg.union(new, id);
    }
}

