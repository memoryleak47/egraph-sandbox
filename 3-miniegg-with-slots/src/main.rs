use lamcalc::*;

mod types;
use types::*;

mod lang;
use lang::*;

mod i_lambda;
use i_lambda::*;

mod i_let;
use i_let::*;

mod syntax;
use syntax::*;

mod slotmap;
use slotmap::*;

mod debug;

mod egraph;
use egraph::*;

mod extract;
use extract::*;

mod rewrite;
use rewrite::*;

mod subst;
use subst::*;

mod small_step;
use small_step::*;

mod tst;
use tst::*;

use std::hash::Hash;
use std::fmt::Debug;
pub type HashMap<K, V> = fnv::FnvHashMap<K, V>;
pub type HashSet<T> = fnv::FnvHashSet<T>;

fn translate(re: RecExpr<ENode>) -> RecExpr<LetENode> {
    let mut out = RecExpr::empty();
    for x in re.node_dag {
        let x = match x {
            ENode::Var(x) => LetENode::Var(x),
            ENode::App(l, r) => LetENode::App(l, r),
            ENode::Lam(x, b) => LetENode::Lam(x, b),
        };
        out.push(x);
    }
    out
}

fn main() {
    let s = app(app(add(), num(0)), num(1));
    check_simplify::<LetExpr>(&s, 16);
}
