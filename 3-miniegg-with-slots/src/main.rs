mod lang;
use lang::*;

mod egraph;
use egraph::*;

mod syntax;
use syntax::*;

/*

mod rewrite;
use rewrite::*;

mod subst;
use subst::*;

mod extract;
use extract::*;
*/

use std::collections::{HashMap, HashSet};

fn main() {
    let s = "(app (lam x (app x x)) (lam y y))";
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
