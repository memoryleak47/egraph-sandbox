mod lang;
use lang::*;

mod ast;
use ast::*;

mod syntax;
use syntax::*;

mod slotmap;
use slotmap::*;

mod debug;


mod egraph;
use egraph::*;

mod extract;
use extract::*;

/*

mod rewrite;
use rewrite::*;

mod subst;
use subst::*;
*/

use std::collections::{HashMap, HashSet};

fn main() {
    // let s = "(app (lam x (app x x)) (lam y y))";
    let s = "(lam x (lam y (app x y)))";
    let re = parse(s);
    let mut eg = EGraph::new();
    let i = eg.add_expr(re.clone());
    dbg!(&eg, &i);
/*

    for _ in 0..10 {
        rewrite_step(&mut eg);
    }

*/
    let re = extract(i, &eg);
    let s = to_string(re);
    println!("{}", s);
}

#[test]
fn test_egraph_roundtrip() {
    let programs = [
        "(lam x0 x0)",
        "(lam x0 (lam x1 x0))",
        "(lam x0 (lam x1 x1))",
        "(lam x0 (lam x1 (app x0 x1)))",
        "(app (lam x0 (app x0 x0)) (lam x1 (app x1 x1)))",
    ];

    for p in programs {
        let re = parse(p);
        let mut eg = EGraph::new();
        let i = eg.add_expr(re.clone());
        let re = extract(i, &eg);
        let s = to_string(re);
        assert_eq!(p, s);
    }
}
