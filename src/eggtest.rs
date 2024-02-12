use egg::*;

define_language! {
    enum Term {
        "lam" = Abstraction([Id; 2]), // TODO the left arg of `lam` should only be a variable, not a full-blown Term.
        "app" = Application([Id; 2]),
        Symb(Symbol),

        "+" = Add([Id; 2]),
        "*" = Mul([Id; 2]),
        Num(i32),
    }
}

fn make_rules() -> Vec<Rewrite<Term, ()>> {
    vec![
        rewrite!("beta-reduction"; "(app (lam ?v ?b) ?c)" => { BetaReduction }),
        rewrite!("mul-0"; "(* ?a 0)" => "0"),
        rewrite!("mul-1"; "(* ?a 1)" => "?a"),
        rewrite!("mul-comm"; "(* ?a ?b)" => "(* ?b ?a)"),
        rewrite!("mul-assoc"; "(* ?a (* ?b ?c))" => "(* (* ?a ?b) ?c)"),

        rewrite!("add-0"; "(+ ?a 0)" => "?a"),
        rewrite!("add-comm"; "(+ ?a ?b)" => "(+ ?b ?a)"),
        rewrite!("add-assoc"; "(+ ?a (+ ?b ?c))" => "(+ (+ ?a ?b) ?c)"),

        rewrite!("distr"; "(* (+ ?a ?b) ?c)" => "(+ (* ?a ?c) (* ?b ?c))"),
    ]
}

struct BetaReduction;

// returns b[v/c]
fn substitute(v: Id, b: Id, c: Id, eg: &mut EGraph<Term, ()>, touched: &mut Vec<Id>) -> Id {
    // TODO this "choice" might be suboptimal!
    let l = eg[b].nodes.iter().next().unwrap().clone();

    let mut map = |[l, r]: [Id; 2], op: fn([Id; 2]) -> Term, eg: &mut EGraph<Term, ()>| {
        let l = substitute(v, l, c, eg, touched);
        let r = substitute(v, l, c, eg, touched);
        let ret = eg.add(op([l, r]));
        touched.push(ret);
        ret
    };

    match l {
        Term::Abstraction([v2, y]) if eg.find(v2) == eg.find(v) => b,
        Term::Abstraction([v2, y]) => {
            let id = substitute(v, y, c, eg, touched);
            let ret = eg.add(Term::Abstraction([v2, id]));
            touched.push(ret);
            ret
        }
        Term::Application(l) => map(l, Term::Application, eg),
        Term::Symb(v2) if eg.find(v) == eg.find(b) => c,
        Term::Symb(v2) => b,
        Term::Add(l) => map(l, Term::Add, eg),
        Term::Mul(l) => map(l, Term::Mul, eg),
        Term::Num(_) => b,
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
    assert_eq!(simplify("(app (lam v b) c)"), "b");
    assert_eq!(simplify("(app (lam v v) c)"), "c");

    let omega = "(lam x (app x x))";
    let infinite_loop = format!("(app {omega} {omega})");

    let id = "(lam x x)";
    let t = "(lam x (lam y x))";
    let s = format!("(app (app {t} {id}) {infinite_loop})");

    assert_eq!(simplify(&s), "(lam x x)");
    assert_eq!(simplify(&infinite_loop), infinite_loop);

    assert_eq!(simplify("(+ x (* 2 0))"), "x");
}
