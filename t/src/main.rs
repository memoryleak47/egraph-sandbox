extern crate miniegg_with_slots;
extern crate explicit_subst;

use miniegg_with_slots::LetExpr;
use explicit_subst::Expr;
use lamcalc::*;

fn main() {
    let g = lamcalc::generate(40).to_string();
    check_simplify::<LetExpr>(&g, 20);
    check_simplify::<Expr>(&g, 20);
}
