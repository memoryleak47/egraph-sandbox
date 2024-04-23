/*
use crate::*;

pub trait RewriteStep {
    fn rewrite_step(eg: &mut EGraph<ENode>);
}

pub struct Expr<T: RewriteStep>(RecExpr<ENode>, std::marker::PhantomData<T>);

impl<T> Realization for Expr<T> where T: RewriteStep {
    type Id = Id;

    fn to_ast_string(&self) -> String {
        self.0.to_string()
    }

    fn from_ast(ast: &Ast) -> Self {
        Expr(RecExpr::parse(&ast.to_string()), std::marker::PhantomData)
    }

    fn simplify(&self, steps: u32) -> Self {
        let mut eg = EGraph::<ENode>::new();
        let i = eg.add_expr(self.0.clone());

        eg.check();
        for _ in 0..steps {
            T::rewrite_step(&mut eg);
            eg.check();
        }

        let re = ast_size_extract(i, &eg);
        Self(re, std::marker::PhantomData::<T>)
    }

    fn find_eq(&self, other: &Self, steps: u32) -> bool {
        let mut eg = EGraph::<ENode>::new();

        let i1 = eg.add_expr(self.0.clone());
        let i2 = eg.add_expr(other.0.clone());

        eg.check();
        for _ in 0..steps {
            T::rewrite_step(&mut eg);
            eg.check();

            if eg.find_id(i1) == eg.find_id(i2) {
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
        fn rewrite_step(eg: &mut EGraph<ENode>) {
            rewrite_step(eg)
        }
    }

    // TODO re-enable. It is currently commented out, as it times out if given too many iterations.
    // lamcalc::unpack_tests!(Expr<BigStep>);
}
pub use big_step::*;

mod small_step {
    use crate::*;

    pub struct SmallStep;

    impl RewriteStep for SmallStep {
        fn rewrite_step(eg: &mut EGraph<ENode>) {
            rewrite_small_step(eg)
        }
    }

    lamcalc::unpack_tests!(Expr<SmallStep>);
}
pub use small_step::*;
*/
