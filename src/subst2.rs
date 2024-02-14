use crate::*;

use std::collections::HashMap;

pub fn subst2() -> Rewrite<Term, ()> {
    rewrite!("beta-reduction2"; "(app (lam ?v ?b) ?c)" => { BetaReduction2 })
}

struct BetaReduction2;

// returns b[v/c]
fn substitute(v: Id, b: Id, c: Id, eg: &mut EGraph<Term, ()>, touched: &mut Vec<Id>) -> Id {
    substitute_impl(v, b, c, eg, touched, &mut Default::default())
}

// returns b[v/c].
// `map` caches the b -> b[v/c] mapping.
fn substitute_impl(v: Id, b: Id, c: Id, eg: &mut EGraph<Term, ()>, touched: &mut Vec<Id>, map: &mut HashMap<Id, Id>) -> Id {
    if let Some(o) = map.get(&b) {
        return *o;
    }

    let mut new_b = None;

    let nodes = eg[b].nodes.clone();

    for x in nodes {
        let i = term_subst(x, v, b, c, eg, touched, map);
        touched.push(i); // TODO not necessarily touched!

        // TODO adding this so late to `map` might cause infinite recursion.
        // Could be solved by allocating a class for `b[v/c]` at the beginning of this function, if that would be possible.
        if new_b.is_none() {
            map.insert(b, i);
            new_b = Some(i);
        }

        // merge the classes created by different enodes from the same original class.
        if let Some(new_b2) = new_b {
            eg.union(i, new_b2);
        }
    }

    new_b.unwrap()
}

fn term_subst(term: Term, v: Id, b: Id, c: Id, eg: &mut EGraph<Term, ()>, touched: &mut Vec<Id>, map: &mut HashMap<Id, Id>) -> Id {
    let alloc = |t, eg: &mut EGraph<Term, ()>, touched: &mut Vec<Id>| {
        let i = eg.add(t);
        touched.push(i);
        i
    };

    match term {
        Term::Abstraction([v2, _]) if eg.find(v2) == eg.find(v) => b,
        Term::Abstraction([v2, y]) => {
            let id = substitute_impl(v, y, c, eg, touched, map);
            alloc(Term::Abstraction([v2, id]), eg, touched)
        }
        Term::Application([l, r]) => {
            let l = substitute_impl(v, l, c, eg, touched, map);
            let r = substitute_impl(v, r, c, eg, touched, map);
            alloc(Term::Application([l, r]), eg, touched)
        },
        Term::Symb(_) if eg.find(v) == eg.find(b) => c,
        Term::Symb(_) => b,
        Term::Add([l, r]) => {
            let l = substitute_impl(v, l, c, eg, touched, map);
            let r = substitute_impl(v, r, c, eg, touched, map);
            alloc(Term::Add([l, r]), eg, touched)
        },
        Term::Mul([l, r]) => {
            let l = substitute_impl(v, l, c, eg, touched, map);
            let r = substitute_impl(v, r, c, eg, touched, map);
            alloc(Term::Mul([l, r]), eg, touched)
        },
        Term::Num(_) => b,
    }
}

impl Applier<Term, ()> for BetaReduction2 {
    fn apply_one(&self, eg: &mut EGraph<Term, ()>, id: Id, subst: &Subst, _pat: Option<&PatternAst<Term>>, _rule_name: Symbol) -> Vec<Id> {
        let v: Var = "?v".parse().unwrap();
        let b: Var = "?b".parse().unwrap();
        let c: Var = "?c".parse().unwrap();

        let v: Id = subst[v];
        let b: Id = subst[b];
        let c: Id = subst[c];

        let mut touched = vec![id];
        let new = substitute(v, b, c, eg, &mut touched);
        eg.union(new, id);

        touched
    }
}

