use crate::*;

pub trait RewriteStep {
    fn rewrite_step(eg: &mut EGraph);
}

pub struct Expr<T: RewriteStep>(RecExpr, std::marker::PhantomData<T>);

impl<T> Realization for Expr<T> where T: RewriteStep {
    fn to_ast_string(&self) -> String {
        self.0.to_string()
    }

    fn from_ast(ast: &Ast) -> Self {
        Expr(RecExpr::parse(&ast.to_string()), std::marker::PhantomData)
    }

    fn simplify(&self, steps: u32) -> Self {
        let mut eg = EGraph::new();
        let i = eg.add_expr(self.0.clone());

        eg.inv();
        for _ in 0..steps {
            T::rewrite_step(&mut eg);
            eg.inv();
        }

        let re = extract(i, &eg);
        Self(re, std::marker::PhantomData::<T>)
    }

    fn find_eq(&self, other: &Self, steps: u32) -> bool {
        let mut eg = EGraph::new();

        let i1 = eg.add_expr(self.0.clone());
        let i2 = eg.add_expr(other.0.clone());

        eg.inv();
        for _ in 0..steps {
            T::rewrite_step(&mut eg);
            eg.inv();

            if eg.normalize_id_by_unionfind(i1) == eg.normalize_id_by_unionfind(i2) {
                return true;
            }
        }

        false
    }
    
}

mod big_step {
    use crate::*;

    pub struct BigStep;

    impl RewriteStep for BigStep {
        fn rewrite_step(eg: &mut EGraph) {
            rewrite_step(eg)
        }
    }

    lamcalc::unpack_tests!(Expr<BigStep>);
}
pub use big_step::*;

mod small_step {
    use crate::*;

    pub struct SmallStep;

    impl RewriteStep for SmallStep {
        fn rewrite_step(eg: &mut EGraph) {
            rewrite_small_step(eg)
        }
    }

    lamcalc::unpack_tests!(Expr<SmallStep>);
}
pub use small_step::*;
