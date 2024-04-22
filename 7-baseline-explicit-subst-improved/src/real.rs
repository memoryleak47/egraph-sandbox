use crate::*;

pub struct Expr(RecExpr<Lambda>);

impl Realization for Expr {
    fn to_ast_string(&self) -> String {
        let mut strings = Vec::new();
        for n in self.0.as_ref() {
            strings.push(match n {
                Lambda::Lambda([x, b]) => format!("(lam {} {})", &strings[usize::from(*x)], &strings[usize::from(*b)]),
                Lambda::App([l, r]) => format!("(app {} {})", &strings[usize::from(*l)], &strings[usize::from(*r)]),
                Lambda::Var(v) => format!("{}", &strings[usize::from(*v)]),
                Lambda::Symbol(s) => format!("{}", s),
                _ => panic!(),
            });
        }

        strings.pop().unwrap()
    }

    fn from_ast(a: &Ast) -> Self {
        let re: RecExpr<Lambda> = to_string(a).parse().unwrap();
        return Self(re);

        fn to_string(a: &Ast) -> String {
            match a {
                Ast::Var(x) => format!("(var {})", x),
                Ast::Lam(x, b) => format!("(lam {} {})", &x, to_string(&*b)),
                Ast::App(l, r) => format!("(app {} {})", to_string(&*l), to_string(&*r)),
            }
        }
    }
    
    fn simplify(&self, steps: u32) -> Self {
        let rewrites = rules();
        println!("egg:");
        let h = |r: &mut Runner<_, _>| {
            println!("{}", r.egraph.total_size());
            println!("varcount: {}", varcount(&r.egraph));
            Ok(())
        };
        let runner = Runner::default()
                        .with_iter_limit(steps as usize)
                        .with_time_limit(std::time::Duration::from_secs(60))
                        .with_node_limit(100000000)
                        .with_scheduler(SimpleScheduler)
                        .with_hook(h)
                        .with_expr(&self.0)
                        .run(&rewrites);
        dbg!(runner.stop_reason);

        let extr = Extractor::new(&runner.egraph, RestrictedAstSize);
        let (_, out) = extr.find_best(runner.roots[0]);

        Self(out)
    }

    fn find_eq(&self, other: &Self, steps: u32) -> bool {
        let rewrites = rules();
        let mut eg = EGraph::default();

        let i1 = eg.add_expr(&self.0);
        let i2 = eg.add_expr(&other.0);
        
        let runner = Runner::default().with_iter_limit(steps as usize).with_scheduler(SimpleScheduler).with_egraph(eg).run(&rewrites);

        runner.egraph.find(i1) == runner.egraph.find(i2)
    }
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

unpack_tests!(Expr);

