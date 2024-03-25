use egg::*;
use lamcalc::*;

mod beta;
use beta::*;

mod cost;
use cost::*;

mod translate;
use translate::*;

mod tst;
use tst::*;

mod varbound;
use varbound::*;

use std::collections::{HashSet, HashMap};

define_language! {
    pub enum ENode {
        "lam" = Lam(Id),
        "app" = App([Id; 2]),
        Var(u32),
        "placeholder" = Placeholder(Id), // contains a Var(i) to express the number.
    }
}

pub type EG = EGraph<ENode, Varbound>;

fn main() {
    let s = "(lam f (lam arg arg))";
    let s = app(y(), String::from(s));
    let s = named_to_de_bruijn(&s);
    let s: RecExpr<ENode> = s.parse().unwrap();

    let rewrites = [beta_reduction()];
    let runner = Runner::default().with_iter_limit(14).with_expr(&s).run(&rewrites);

    let extr = Extractor::new(&runner.egraph, MyAstSize);
    let (_, out) = extr.find_best(runner.roots[0]);

    dbg!(out.to_string());

}
