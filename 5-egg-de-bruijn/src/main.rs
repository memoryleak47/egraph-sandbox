use egg::*;

mod ast;
use ast::*;

mod beta;
use beta::*;

mod cost;
use cost::*;

mod translate;

use std::collections::{HashSet, HashMap};

define_language! {
    pub enum ENode {
        "lam" = Lam(Id),
        "app" = App([Id; 2]),
        Var(u32),
        "placeholder" = Placeholder(Id), // contains a Var(i) to express the number.
    }
}


pub type EG = EGraph<ENode, ()>;

fn main() {
    let mut eg = EG::new(());
    let s = "(lam 0)";
    let s: RecExpr<ENode> = format!("(app {} {})", s, s).parse().unwrap();

    let rewrites = [beta_reduction()];
    let runner = Runner::default().with_iter_limit(10).with_expr(&s).run(&rewrites);

    let mut extr = Extractor::new(&runner.egraph, MyAstSize);
    let (_, out) = extr.find_best(runner.roots[0]);

    dbg!(out.to_string());
}
