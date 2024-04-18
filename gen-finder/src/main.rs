extern crate miniegg_with_slots;
extern crate explicit_subst;

use miniegg_with_slots::LetExpr;
use explicit_subst::Expr;
use lamcalc::*;

fn main() {
    // let g = lamcalc::generate(40).to_string();
    let g = "(app (app (app (lam b (lam b (app b (lam z b)))) (app (lam z z) (lam c c))) (lam y (app y (app y (lam b b))))) (lam c (lam y (app (lam b (app b (lam y (app (lam c c) c)))) (lam c (app (lam c (lam x x)) (lam a (lam a (app a (lam y a))))))))))";
    let g = &format!("(app {} {})", g, g);
    dbg!(&g);
    let a = simplify::<LetExpr>(&g, 20);
    dbg!(&a);
    let b = simplify::<Expr>(&g, 15); // It runs out of memory at 16 iterations.
    dbg!(&b);
}
