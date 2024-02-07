use egg::*;

define_language! {
    enum SimpleLanguage {
        "abs" = Abstraction([Id; 2]),
        "app" = Application([Id; 2]),
        Var(Symbol),
    }
}

fn make_rules() -> Vec<Rewrite<SimpleLanguage, ()>> {
    vec![
        rewrite!("beta-reduction"; "(app (abs ?a ?b) ?c)" => { BetaReduction }),
    ]
}

struct BetaReduction;

impl Applier<SimpleLanguage, ()> for BetaReduction {
    fn apply_one(&self, eg: &mut EGraph<SimpleLanguage, ()>, id: Id, subst: &Subst, pat: Option<&PatternAst<SimpleLanguage>>, _rule_name: Symbol) -> Vec<Id> {
        let a: Var = "?a".parse().unwrap();
        let b: Var = "?b".parse().unwrap();
        let c: Var = "?c".parse().unwrap();

        let a = subst.get(a).unwrap().clone();
        let b = subst.get(b).unwrap().clone();
        let c = subst.get(c).unwrap().clone();

        eg.union(id, b);

        vec![id, b]
    }
}

fn simplify(s: &str) -> String {
    let expr: RecExpr<SimpleLanguage> = s.parse().unwrap();
    let runner = Runner::default().with_expr(&expr).run(&make_rules());
    let root = runner.roots[0];
    let extractor = Extractor::new(&runner.egraph, AstSize);
    let (best_cost, best) = extractor.find_best(root);
    println!("Simplified {} to {} with cost {}", expr, best, best_cost);
    best.to_string()
}

pub fn main() {
    assert_eq!(simplify("(app (abs a b) c)"), "b");
}
