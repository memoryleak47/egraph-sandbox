use egg::Id;
use egg::Symbol;

use crate::lang;

egg::define_language! {
    pub enum Term {
        Abstraction(Symbol, Id),
        "app" = Application([Id; 2]),
        Var(Symbol),
    }
}

fn make_rules() -> Vec<egg::Rewrite<Term, ()>> {
    vec![]
}

fn simplify(s: &str) -> String {
    let expr: egg::RecExpr<Term> = s.parse().unwrap();
    let runner = egg::Runner::default().with_expr(&expr).run(&make_rules());

    let root = runner.roots[0];

    let extractor = egg::Extractor::new(&runner.egraph, egg::AstSize);
    let (best_cost, best) = extractor.find_best(root);
    best.to_string()
}
