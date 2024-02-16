use crate::*;

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

//// The beta-reduction2 Rewrite Rule:

pub fn subst2() -> Rewrite<Term, ()> {
    rewrite!("beta-reduction2"; "(app (lam ?x ?b) ?t)" => { BetaReduction })
}

struct BetaReduction;

impl Applier<Term, ()> for BetaReduction {
    fn apply_one(&self, eg: &mut EGraph<Term, ()>, id: Id, subst: &Subst, _pat: Option<&PatternAst<Term>>, _rule_name: Symbol) -> Vec<Id> {
        let b: Var = "?b".parse().unwrap();
        let x: Var = "?x".parse().unwrap();
        let t: Var = "?t".parse().unwrap();

        let b: Id = subst[b];
        let x: Id = subst[x];
        let t: Id = subst[t];

        let mut touched = vec![id];
        let new = substitute(b, x, t, eg, &mut touched);
        eg.union(new, id);

        touched
    }
}


//// The Substitution implementation:

// returns b[x := t]
fn substitute(b: Id, x: Id, t: Id, eg: &mut EGraph<Term, ()>, touched: &mut Vec<Id>) -> Id {
    substitute_impl(b, x, t, eg, touched, &mut Default::default())
}

// returns b[x := t].
// `map` caches the b -> b[x := t] mapping.
fn substitute_impl(b: Id, x: Id, t: Id, eg: &mut EGraph<Term, ()>, touched: &mut Vec<Id>, map: &mut HashMap<Id, Id>) -> Id {
    if let Some(o) = map.get(&b) {
        return *o;
    }

    let new_b = alloc_eclass(eg, touched);
    map.insert(b, new_b);

    for enode in eg[b].nodes.clone() {
        if let Term::Placeholder(_) = enode { continue; }

        let id = enode_subst(enode, b, x, t, eg, touched, map);
        eg.union(new_b, id);

        touched.push(id);
    }

    new_b
}

// `enode` is an enode from the eclass `b`.
// we return an eclass containing `enode[x := t]`
fn enode_subst(enode: Term, b: Id, x: Id, t: Id, eg: &mut EGraph<Term, ()>, touched: &mut Vec<Id>, map: &mut HashMap<Id, Id>) -> Id {
    let alloc = |t, eg: &mut EGraph<Term, ()>, touched: &mut Vec<Id>| {
        let i = eg.add(t);
        touched.push(i);
        i
    };

    match enode {
        // (lam x2 b2)[x := t] --> (lam x2 b2), if x = x2.
        // In other words, we don't change anything, if x gets re-bound.
        Term::Abstraction([x2, _]) if eg.find(x2) == eg.find(x) => b,

        // (lam x2 b2)[x := t] --> (lam x2 b2[x := t]), if x != x2.
        Term::Abstraction([x2, b2]) => {
            let b2 = substitute_impl(b2, x, t, eg, touched, map);
            alloc(Term::Abstraction([x2, b2]), eg, touched)
        }

        // (app l r)[x := t] --> (app l[x := t] r[x := t])
        Term::Application([l, r]) => {
            let l = substitute_impl(l, x, t, eg, touched, map);
            let r = substitute_impl(r, x, t, eg, touched, map);
            alloc(Term::Application([l, r]), eg, touched)
        },

        // x2[x := t] --> t, if x = x2.
        Term::Symb(_) if eg.find(x) == eg.find(b) => t,

        // x2[x := t] --> x2, if x != x2.
        Term::Symb(_) => b,

        // similar to `app`.
        Term::Add([l, r]) => {
            let l = substitute_impl(l, x, t, eg, touched, map);
            let r = substitute_impl(r, x, t, eg, touched, map);
            alloc(Term::Add([l, r]), eg, touched)
        },

        // similar to `app`.
        Term::Mul([l, r]) => {
            let l = substitute_impl(l, x, t, eg, touched, map);
            let r = substitute_impl(r, x, t, eg, touched, map);
            alloc(Term::Mul([l, r]), eg, touched)
        },

        // n[x := t] --> n, numbers ignore substitution.
        Term::Num(_) => b,
        Term::Placeholder(_) => panic!("can't substitute in a Placeholder!"),
    }
}

// allocates a new (conceptually empty) eclass, by doing eg.add(Placeholder(GLOBAL_CTR++)).
fn alloc_eclass(eg: &mut EGraph<Term, ()>, touched: &mut Vec<Id>) -> Id {
    static GLOBAL_CTR: AtomicUsize = AtomicUsize::new(0);
    let num = GLOBAL_CTR.fetch_add(1, Ordering::SeqCst);

    let num = eg.add(Term::Num(num as i32));
    touched.push(num);
    let num = eg.add(Term::Placeholder(num));
    touched.push(num);
    num
}
