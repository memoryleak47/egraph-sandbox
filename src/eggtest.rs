use egg::*;

define_language! {
    enum Term {
        "abs" = Abstraction([Id; 2]),
        "app" = Application([Id; 2]),
        Symb(Symbol),
    }
}

fn make_rules() -> Vec<Rewrite<Term, ()>> {
    vec![
        rewrite!("beta-reduction"; "(app (abs ?v ?b) ?c)" => { BetaReduction }),
    ]
}

struct BetaReduction;

// returns b[v/c]
fn substitute(v: Id, b: Id, c: Id, eg: &mut EGraph<Term, ()>, touched: &mut Vec<Id>) -> Id {
    // TODO this "choice" might be suboptimal!
    let l = eg[b].nodes.iter().next().unwrap().clone();

    match l {
        Term::Abstraction([v2, y]) if eg.find(v2) == eg.find(v) => b,
        Term::Abstraction([v2, y]) => {
            let id = substitute(v, y, c, eg, touched);
            let ret = eg.add(Term::Abstraction([v2, id]));
            touched.push(ret);
            ret
        }
        Term::Application([l, r]) => {
            let l = substitute(v, l, c, eg, touched);
            let r = substitute(v, l, c, eg, touched);
            let ret = eg.add(Term::Application([l, r]));
            touched.push(ret);
            ret
        },
        Term::Symb(v2) if eg.find(v) == eg.find(b) => c,
        Term::Symb(v2) => b,
    }
}

impl Applier<Term, ()> for BetaReduction {
    fn apply_one(&self, eg: &mut EGraph<Term, ()>, id: Id, subst: &Subst, pat: Option<&PatternAst<Term>>, _rule_name: Symbol) -> Vec<Id> {
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

fn simplify(s: &str) -> String {
    let expr: RecExpr<Term> = s.parse().unwrap();
    let runner = Runner::default().with_expr(&expr).run(&make_rules());
    let root = runner.roots[0];
    let extractor = Extractor::new(&runner.egraph, AstSize);
    let (best_cost, best) = extractor.find_best(root);
    println!("Simplified {} to {} with cost {}", expr, best, best_cost);
    best.to_string()
}

pub fn main() {
    assert_eq!(simplify("(app (abs v b) c)"), "b");
    assert_eq!(simplify("(app (abs v v) c)"), "c");
    assert_eq!(simplify("(app (abs v v) c)"), "c");
}
