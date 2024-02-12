use crate::*;

pub struct BetaReduction1;

// returns b[v/c]
fn substitute(v: Id, b: Id, c: Id, eg: &mut EGraph<Term, ()>, touched: &mut Vec<Id>) -> Id {
    // TODO this "choice" might be suboptimal!
    let l = eg[b].nodes.iter().next().unwrap().clone();

    let mut map = |[l, r]: [Id; 2], op: fn([Id; 2]) -> Term, eg: &mut EGraph<Term, ()>| {
        let l = substitute(v, l, c, eg, touched);
        let r = substitute(v, r, c, eg, touched);
        let ret = eg.add(op([l, r]));
        touched.push(ret);
        ret
    };

    match l {
        Term::Abstraction([v2, _]) if eg.find(v2) == eg.find(v) => b,
        Term::Abstraction([v2, y]) => {
            let id = substitute(v, y, c, eg, touched);
            let ret = eg.add(Term::Abstraction([v2, id]));
            touched.push(ret);
            ret
        }
        Term::Application(l) => map(l, Term::Application, eg),
        Term::Symb(_) if eg.find(v) == eg.find(b) => c,
        Term::Symb(_) => b,
        Term::Add(l) => map(l, Term::Add, eg),
        Term::Mul(l) => map(l, Term::Mul, eg),
        Term::Num(_) => b,
    }
}

impl Applier<Term, ()> for BetaReduction1 {
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

