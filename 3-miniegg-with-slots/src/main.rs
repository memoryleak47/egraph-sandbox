mod lang;
use lang::*;

mod ast;
use ast::*;

mod syntax;
use syntax::*;

mod slotmap;
use slotmap::*;

/*

mod egraph;
use egraph::*;

mod rewrite;
use rewrite::*;

mod subst;
use subst::*;

mod extract;
use extract::*;
*/

use std::collections::{HashMap, HashSet};

fn main() {
    // let s = "(app (lam x (app x x)) (lam y y))";
    let s = "(lam x x)";
    let re = parse(s);
/*
    let mut eg = EGraph::new();
    let i = eg.add_expr(re);

    for _ in 0..10 {
        rewrite_step(&mut eg);
    }

    let re = extract(i, &eg);
*/
    let s = to_string(re);
    println!("{}", s);
}
