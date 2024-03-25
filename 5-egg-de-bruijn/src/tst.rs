use crate::*;

pub struct Expr(RecExpr<ENode>);

impl Realization for Expr {
    fn to_ast_string(&self) -> String {
        de_bruijn_to_named(&self.0.to_string())
    }

    fn from_ast(ast: &Ast) -> Self {
        let s = ast.to_string();
        let s = named_to_de_bruijn(&s);
        let s: RecExpr<ENode> = s.parse().unwrap();
        let s = Expr(s);
        s
    }

    fn simplify(&self, steps: u32) -> Self {
        let rewrites = [beta_reduction()];
        let runner = Runner::default().with_iter_limit(steps as usize).with_expr(&self.0).run(&rewrites);

        let mut extr = Extractor::new(&runner.egraph, MyAstSize);
        let (_, out) = extr.find_best(runner.roots[0]);

        Expr(out)
    }

    fn find_eq(&self, other: &Self, steps: u32) -> bool {
        let mut eg = EG::new(());

        let i1 = eg.add_expr(&self.0);
        let i2 = eg.add_expr(&other.0);

        let rewrites = [beta_reduction()];
        let runner = Runner::default().with_iter_limit(steps as usize).with_egraph(eg).run(&rewrites);

        runner.egraph.find(i1) == runner.egraph.find(i2)
    }
}

unpack_tests!(Expr);
