use lamcalc::*;
use egg::*;

mod lambda;
use lambda::*;

mod cost;
use cost::*;

struct Expr(RecExpr<Lambda>);

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

        let h = |r: &mut Runner<_, _>| {
            // println!("{}", r.egraph.total_size());
            Ok(())
        };
        let runner = Runner::default()
                                    .with_iter_limit(steps as usize)
                                    .with_scheduler(SimpleScheduler)
                                    .with_node_limit(10000000000000)
                                    .with_time_limit(std::time::Duration::from_secs(60*60))
                                    .with_expr(&self.0)
                                    .with_hook(h)
                                    .run(&rewrites);
        // println!("last: {}", runner.egraph.total_size());

        let extr = Extractor::new(&runner.egraph, RestrictedAstSize);
        let (_, out) = extr.find_best(runner.roots[0]);

        Self(out)
    }

    fn find_eq(&self, other: &Self, steps: u32) -> bool {
        let rewrites = rules();
        let mut eg = EGraph::default();

        let i1 = eg.add_expr(&self.0);
        let i2 = eg.add_expr(&other.0);
        
        let h = |r: &mut Runner<_, _>| {
            // println!("{}", r.egraph.total_size());
            Ok(())
        };
        let runner = Runner::default()
                                    .with_iter_limit(steps as usize)
                                    .with_scheduler(SimpleScheduler)
                                    .with_node_limit(10000000000000)
                                    .with_time_limit(std::time::Duration::from_secs(60*60))
                                    .with_egraph(eg)
                                    .with_hook(h)
                                    .run(&rewrites);

        runner.egraph.find(i1) == runner.egraph.find(i2)
    }
}

unpack_tests!(Expr);

fn main() {
    let s = app(app(add(), num(2)), num(2));
    check_simplify::<Expr>(&s, 24);

}
