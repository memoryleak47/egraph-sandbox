use crate::*;

struct LetExpr(RecExpr<LetENode>);

impl Realization for LetExpr {
    fn to_ast_string(&self) -> String {
        from_let(&self.0).to_string()
    }

    fn from_ast(ast: &Ast) -> Self {
        let re = RecExpr::parse(&ast.to_string());
        let re = to_let(&re);
        LetExpr(re)
    }

    fn simplify(&self, steps: u32) -> Self {
        let mut eg = EGraph::<LetENode>::new();
        let i = eg.add_expr(self.0.clone());

        eg.inv();
        for _ in 0..steps {
            rewrite_let(&mut eg);
            eg.inv();
        }

        let re = extract(i, &eg);
        Self(re)
    }

    fn find_eq(&self, other: &Self, steps: u32) -> bool {
        let mut eg = EGraph::<LetENode>::new();

        let i1 = eg.add_expr(self.0.clone());
        let i2 = eg.add_expr(other.0.clone());

        eg.inv();
        for _ in 0..steps {
            rewrite_let(&mut eg);
            eg.inv();

            if eg.find_id(i1) == eg.find_id(i2) {
                return true;
            }
        }

        false
    }
    
}

fn to_let(re: &RecExpr<ENode>) -> RecExpr<LetENode> {
    let mut out = RecExpr::empty();
    for x in re.node_dag.clone() {
        let x = match x {
            ENode::Var(x) => LetENode::Var(x),
            ENode::App(l, r) => LetENode::App(l, r),
            ENode::Lam(x, b) => LetENode::Lam(x, b),
        };
        out.push(x);
    }
    out
}

fn from_let(re: &RecExpr<LetENode>) -> RecExpr<ENode> {
    let mut out = RecExpr::empty();
    for x in re.node_dag.clone() {
        let x = match x {
            LetENode::Var(x) => ENode::Var(x),
            LetENode::App(l, r) => ENode::App(l, r),
            LetENode::Lam(x, b) => ENode::Lam(x, b),
            LetENode::Let(..) => panic!("it contains let!"),
        };
        out.push(x);
    }
    out
}

lamcalc::unpack_tests!(LetExpr);

