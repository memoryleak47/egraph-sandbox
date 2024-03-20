use crate::*;

mod build;
use build::*;

mod helper;
use helper::*;

#[test]
fn cannot_simplify() {
    let s = [
        "(lam x0 x0)",
        "(lam x0 (lam x1 x0))",
        "(lam x0 (lam x1 x1))",
        "(lam x0 (lam x1 (app x0 x1)))",
    ];

    for p in s {
        let out = simplify(p, 10);
        assert_alpha_eq(&*out, p);
    }
}

#[test]
fn nested_identity1() {
    let p = "(app (lam x0 x0) (lam x1 x1))";
    check_simplify(p, 10);
}

#[test]
fn nested_identity2() {
    let p = "(app (lam x0 x0) (lam x1 (app x1 x1)))";
    check_simplify(p, 10);
}

#[test]
fn nested_identity3() {
    let p = "(app (lam x0 (app x0 x0)) (lam x1 x1))";
    check_simplify(p, 10);
}

#[test]
fn simple_beta_test() {
    let p = "(lam x (lam y
        (app
            (lam z (app x z))
        y)
    ))";
    check_simplify(p, 10);
}

#[test]
fn redudant_slot() {
    // y is unused, and hence x is effectively redundant.
    let p = "(lam x (app (lam y (lam z z)) x))";
    check_simplify(p, 10);
}

#[test]
fn add_test() {
    let s = app(app(add(), num(0)), num(1));
    check_simplify(&s, 5);
}

#[test]
fn inf_loop() {
    let p = "(app (lam x0 (app x0 x0)) (lam x1 (app x1 x1)))";
    let out = simplify(p, 3);
    assert_alpha_eq(&out, p);
}

// A y-combinator example that directly yields "f x = x" without looping.
#[test]
fn y_identity() {
    let p = "(lam f (lam arg arg))";
    let s = app(y(), String::from(p));

    let out = simplify(&s, 30);
    assert_alpha_eq(&out, "(lam x x)");
}
