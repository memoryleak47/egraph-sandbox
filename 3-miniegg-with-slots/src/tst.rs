use crate::*;

impl Realization for RecExpr {
    fn to_ast_string(&self) -> String {
        self.to_string()
    }

    fn from_ast(ast: &Ast) -> Self {
        Self::parse(&ast.to_string())
    }

    fn simplify(&self, steps: u32) -> Self {
        let mut eg = EGraph::new();
        let i = eg.add_expr(self.clone());

        eg.inv();
        for _ in 0..steps {
            rewrite_step(&mut eg);
            eg.inv();
        }

        let re = extract(i, &eg);
        re
    }

    fn find_eq(&self, other: &Self, steps: u32) -> bool {
        let mut eg = EGraph::new();

        let i1 = eg.add_expr(self.clone());
        let i2 = eg.add_expr(other.clone());

        eg.inv();
        for _ in 0..steps {
            rewrite_step(&mut eg);
            eg.inv();

            if eg.normalize_id_by_unionfind(i1) == eg.normalize_id_by_unionfind(i2) {
                return true;
            }
        }

        false
    }
    
}

lamcalc::unpack_tests!(RecExpr);
