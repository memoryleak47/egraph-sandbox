use crate::*;

use std::collections::HashMap;

pub fn subst2() -> Rewrite<Term, ()> {
    rewrite!("beta-reduction2"; "(app (lam ?v ?b) ?c)" => { BetaReduction })
}

struct BetaReduction;

// returns b[v/c]
fn substitute(v: Id, b: Id, c: Id, eg: &mut EGraph<Term, ()>, touched: &mut Vec<Id>) -> Id {
    let mut i = 0;
    let mut class_gen = |eg: &mut EGraph<Term, ()>, touched: &mut Vec<Id>| -> Id {
        let orig_i = i;
        i += 1;

        let num = eg.add(Term::Num(orig_i));
        touched.push(num);
        let placeholder = eg.add(Term::Placeholder(num));
        touched.push(placeholder);

        placeholder
    };
    substitute_impl(v, b, c, eg, touched, &mut Default::default(), &mut class_gen)
}

// returns b[v/c].
// `map` caches the b -> b[v/c] mapping.
fn substitute_impl(v: Id, b: Id, c: Id, eg: &mut EGraph<Term, ()>, touched: &mut Vec<Id>, map: &mut HashMap<Id, Id>, class_gen: &mut impl FnMut(&mut EGraph<Term, ()>, &mut Vec<Id>) -> Id) -> Id {
    if let Some(o) = map.get(&b) {
        return *o;
    }

    let new_b = (*class_gen)(eg, touched);
    map.insert(b, new_b);

    let nodes = eg[b].nodes.clone();

    for x in nodes {
        if matches!(x, Term::Placeholder(_)) { continue; }

        let i = term_subst(x, v, b, c, eg, touched, map, class_gen);
        touched.push(i); // TODO not necessarily touched!

        eg.union(i, new_b);
    }

    new_b
}

fn term_subst(term: Term, v: Id, b: Id, c: Id, eg: &mut EGraph<Term, ()>, touched: &mut Vec<Id>, map: &mut HashMap<Id, Id>, class_gen: &mut impl FnMut(&mut EGraph<Term, ()>, &mut Vec<Id>) -> Id) -> Id {
    let alloc = |t, eg: &mut EGraph<Term, ()>, touched: &mut Vec<Id>| {
        let i = eg.add(t);
        touched.push(i);
        i
    };

    match term {
        Term::Abstraction([v2, _]) if eg.find(v2) == eg.find(v) => b,
        Term::Abstraction([v2, y]) => {
            let id = substitute_impl(v, y, c, eg, touched, map, class_gen);
            alloc(Term::Abstraction([v2, id]), eg, touched)
        }
        Term::Application([l, r]) => {
            let l = substitute_impl(v, l, c, eg, touched, map, class_gen);
            let r = substitute_impl(v, r, c, eg, touched, map, class_gen);
            alloc(Term::Application([l, r]), eg, touched)
        },
        Term::Symb(_) if eg.find(v) == eg.find(b) => c,
        Term::Symb(_) => b,
        Term::Add([l, r]) => {
            let l = substitute_impl(v, l, c, eg, touched, map, class_gen);
            let r = substitute_impl(v, r, c, eg, touched, map, class_gen);
            alloc(Term::Add([l, r]), eg, touched)
        },
        Term::Mul([l, r]) => {
            let l = substitute_impl(v, l, c, eg, touched, map, class_gen);
            let r = substitute_impl(v, r, c, eg, touched, map, class_gen);
            alloc(Term::Mul([l, r]), eg, touched)
        },
        Term::Num(_) => b,
        Term::Placeholder(_) => panic!("can't substitute in a Placeholder!"),
    }
}

impl Applier<Term, ()> for BetaReduction {
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

