use lamcalc::*;
use egg::*;

mod lambda;
use lambda::*;

mod cost;
use cost::*;

struct Expr(RecExpr<Lambda>);

impl Realization for Expr {
    // TODO
    fn to_ast_string(&self) -> String {
        self.0.to_string()
    }

    // TODO
    fn from_ast(a: &Ast) -> Self {
        let re: RecExpr<Lambda> = a.to_string().parse().unwrap();
        Self(re)
    }
    
    fn simplify(&self, steps: u32) -> Self {
        let rewrites = rules();
        let runner = Runner::default().with_iter_limit(steps as usize).with_expr(&self.0).run(&rewrites);

        let extr = Extractor::new(&runner.egraph, RestrictedAstSize);
        let (_, out) = extr.find_best(runner.roots[0]);

        Self(out)
    }

    fn find_eq(&self, other: &Self, steps: u32) -> bool {
        let rewrites = rules();
        let mut eg = EGraph::default();

        let i1 = eg.add_expr(&self.0);
        let i2 = eg.add_expr(&other.0);
        
        let runner = Runner::default().with_iter_limit(steps as usize).with_egraph(eg).run(&rewrites);

        runner.egraph.find(i1) == runner.egraph.find(i2)
    }
}

unpack_tests!(Expr);

fn main() {}
