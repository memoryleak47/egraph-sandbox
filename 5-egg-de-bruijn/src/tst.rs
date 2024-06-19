use crate::*;

pub struct DeBruijnReal(EGraph<ENode, Varbound>);

impl Realization for DeBruijnReal {
    type Id = Id;

    fn new() -> Self {
        DeBruijnReal(EGraph::default())
    }

    fn add_ast(&mut self, ast: &Ast) -> Self::Id {
        let s = named_to_de_bruijn(&ast.to_string());
        let re = s.parse().unwrap();
        self.0.add_expr(&re)
    }

    fn extract_ast(&self, id: Self::Id) -> Ast {
        let extr = Extractor::new(&self.0, MyAstSize);
        let (_, out) = extr.find_best(id);

        let s = de_bruijn_to_named(&out.to_string());
        Ast::parse(&s)
    }

    fn find(&self, id: Self::Id) -> Self::Id {
        self.0.find(id)
    }

    fn step(&mut self) {
        let rules = [beta_reduction()];

        let mut eg2 = EGraph::default();
        std::mem::swap(&mut eg2, &mut self.0);
        let mut r = Runner::default().with_egraph(eg2).with_iter_limit(1).run(&rules);
        std::mem::swap(&mut r.egraph, &mut self.0);
    }

    fn enode_count(&self) -> usize { self.0.total_number_of_nodes() }
    fn eclass_count(&self) -> usize { self.0.classes().filter(|x| x.id == self.find(x.id)).count() }
}

unpack_tests!(DeBruijnReal);
