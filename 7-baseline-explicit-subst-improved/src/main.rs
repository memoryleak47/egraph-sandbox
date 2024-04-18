use lamcalc::*;
use egg::*;

mod lambda;
use lambda::*;

mod cost;
use cost::*;

mod real;
use real::*;

fn main() {
    let s = app(app(add(), num(2)), num(2));
    check_simplify::<Expr>(&s, 35);
}
