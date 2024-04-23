use crate::*;

pub struct LetReal(EGraph<LetENode>);

impl Realization for LetReal {
    type Id = Id;

    fn new() -> Self {
        LetReal(EGraph::new())
    }

    fn add_ast(&mut self, ast: &Ast) -> Self::Id {
        let re = RecExpr::<LetENode>::parse(&ast.to_string());
        self.0.add_expr(re)
    }

    fn extract_ast(&self, id: Self::Id) -> Ast {
        let out = extract::<LetENode, AstSizeNoLet>(id, &self.0);
        Ast::parse(&out.to_string())
    }

    fn find(&self, id: Self::Id) -> Self::Id {
        self.0.find_id(id)
    }

    fn step(&mut self) {
        rewrite_let(&mut self.0);
    }

    fn enode_count(&self) -> usize { self.0.total_size() }
    fn eclass_count(&self) -> usize { self.0.ids().len() } 
}

impl RecExpr<LetENode> {
    pub fn to_string(&self) -> String {
        from_let(self).to_string()
    }

    pub fn parse(s: &str) -> Self {
        to_let(&RecExpr::<ENode>::parse(s))
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

lamcalc::unpack_tests!(LetReal);
