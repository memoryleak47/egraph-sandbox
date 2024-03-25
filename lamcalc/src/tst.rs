use crate::*;

pub fn test_all<R: Realization>() {
    test_cannot_simplify::<R>();
    test_nested_identity1::<R>();
    test_nested_identity2::<R>();
    test_nested_identity3::<R>();
    test_simple_beta::<R>();
    test_redudant_slot::<R>();
    test_add::<R>();
    test_add_incomplete::<R>();
    test_inf_loop::<R>();
    test_y_identity::<R>();
    test_add_y_step::<R>();
}

fn test_cannot_simplify<R: Realization>() {
    let s = [
        "(lam x0 x0)",
        "(lam x0 (lam x1 x0))",
        "(lam x0 (lam x1 x1))",
        "(lam x0 (lam x1 (app x0 x1)))",
    ];

    for p in s {
        let out = simplify::<R>(p, 10);
        assert_alpha_eq(&*out, p);
    }
}

fn test_nested_identity1<R: Realization>() {
    let p = "(app (lam x0 x0) (lam x1 x1))";
    check_simplify::<R>(p, 10);
}

fn test_nested_identity2<R: Realization>() {
    let p = "(app (lam x0 x0) (lam x1 (app x1 x1)))";
    check_simplify::<R>(p, 10);
}

fn test_nested_identity3<R: Realization>() {
    let p = "(app (lam x0 (app x0 x0)) (lam x1 x1))";
    check_simplify::<R>(p, 10);
}

fn test_simple_beta<R: Realization>() {
    let p = "(lam x (lam y
        (app
            (lam z (app x z))
        y)
    ))";
    check_simplify::<R>(p, 10);
}

fn test_redudant_slot<R: Realization>() {
    // y is unused, and hence x is effectively redundant.
    let p = "(lam x (app (lam y (lam z z)) x))";
    check_simplify::<R>(p, 10);
}

fn test_add<R: Realization>() {
    let s = app(app(add(), num(0)), num(1));
    check_simplify::<R>(&s, 5);
}

fn test_add_incomplete<R: Realization>() {
    let s = app(app(add(), num(2)), num(3));
    check_simplify_incomplete::<R>(&s, 5);
}

fn test_inf_loop<R: Realization>() {
    let p = "(app (lam x0 (app x0 x0)) (lam x1 (app x1 x1)))";
    let out = simplify::<R>(p, 3);
    assert_alpha_eq(&out, p);
}

// A y-combinator example that directly yields "f x = x" without looping.
fn test_y_identity<R: Realization>() {
    let p = "(lam f (lam arg arg))";
    let s = app(y(), String::from(p));

    let out = simplify::<R>(&s, 30);
    assert_alpha_eq(&out, "(lam x x)");
}

fn test_add_y_step<R: Realization>() {
    let s1 = app(add_impl(), add());
    let s2 = add();
    check_eq::<R>(&s1, &s2, 1);
}
