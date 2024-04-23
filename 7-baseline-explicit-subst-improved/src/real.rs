use crate::*;

pub struct LambdaRealImproved(EGraph<Lambda, LambdaAnalysis>);

impl Realization for LambdaRealImproved {
    type Id = Id;

    fn new() -> Self {
        LambdaRealImproved(EGraph::default())
    }

    fn add_ast(&mut self, ast: &Ast) -> Self::Id {
        let re = re_from_ast(ast);
        self.0.add_expr(&re)
    }

    fn extract_ast(&self, id: Self::Id) -> Ast {
        let extr = Extractor::new(&self.0, RestrictedAstSize);
        let (_, out) = extr.find_best(id);

        re_to_ast(&out)
    }

    fn find(&self, id: Self::Id) -> Self::Id {
        self.0.find(id)
    }

    fn step(&mut self) {
        let mut eg2 = EGraph::default();
        std::mem::swap(&mut eg2, &mut self.0);
        let mut r = Runner::default().with_egraph(eg2).with_iter_limit(1).run(&rules());
        std::mem::swap(&mut r.egraph, &mut self.0);
    }

    fn enode_count(&self) -> usize { self.0.total_size() }
    fn eclass_count(&self) -> usize { self.0.classes().filter(|x| x.id == self.find(x.id)).count() }
}

fn varcount(eg: &EGraph<Lambda, LambdaAnalysis>) -> usize {
    use std::collections::HashSet;

    let mut hashset: HashSet<&Symbol> = HashSet::default();
    for c in eg.classes() {
        for i in c.iter() {
            if let Lambda::Symbol(v) = i {
                hashset.insert(v);
            }
        }
    }

    hashset.len()
}

fn re_to_ast(re: &RecExpr<Lambda>)-> Ast {
    let mut strings = Vec::new();
    for n in re.as_ref() {
        strings.push(match n {
            Lambda::Lambda([x, b]) => format!("(lam {} {})", &strings[usize::from(*x)], &strings[usize::from(*b)]),
            Lambda::App([l, r]) => format!("(app {} {})", &strings[usize::from(*l)], &strings[usize::from(*r)]),
            Lambda::Var(v) => format!("{}", &strings[usize::from(*v)]),
            Lambda::Symbol(s) => format!("{}", s),
            _ => panic!(),
        });
    }

    Ast::parse(strings.last().unwrap())
}

fn re_from_ast(a: &Ast) -> RecExpr<Lambda> {
    return to_string(a).parse().unwrap();

    fn to_string(a: &Ast) -> String {
        match a {
            Ast::Var(x) => format!("(var {})", x),
            Ast::Lam(x, b) => format!("(lam {} {})", &x, to_string(&*b)),
            Ast::App(l, r) => format!("(app {} {})", to_string(&*l), to_string(&*r)),
        }
    }
}
    
unpack_tests!(LambdaRealImproved);
