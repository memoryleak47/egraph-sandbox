use lamcalc::*;
use egg::*;

mod lambda;
use lambda::*;

mod cost;
use cost::*;

mod real;
use real::*;

fn main() {
    let s = "(app (app (app (lam b (lam b (app b (lam z b)))) (app (lam z z) (lam c c))) (lam y (app y (app y (lam b b))))) (lam c (lam y (app (lam b (app b (lam y (app (lam c c) c)))) (lam c (app (lam c (lam x x)) (lam a (lam a (app a (lam y a))))))))))";
    let s = format!("(app {} {})", s, s);
    check_simplify::<LambdaRealImproved>(&s);
}
