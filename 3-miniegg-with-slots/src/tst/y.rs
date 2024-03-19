use crate::tst::*;

// The Y combinator.
fn y() -> String {
    let a = format!("(lam x (app f (app x x)))");

    format!("(lam f (app {a} {a}))")
}

fn zero() -> String {
    format!("(lam x (lam y x))")
}

fn suc() -> String {
    format!("(lam arg (lam x (lam y (app y arg))))")
}

fn num(x: u32) -> String {
    let mut out = zero();
    for _ in 0..x {
        out = app(suc(), out);
    }
    out
}

fn app(x: String, y: String) -> String {
    format!("(app {x} {y})")
}

// add 0 y = y
// add (x+1) y = add x (y+1)
// 
// add = Y add_impl
// 
// add_impl add x y = (x y) (\z. add z (suc y))
fn add() -> String {
    let s = suc();
    let add_impl = format!("(lam add (lam x (lam y
        (app (app x y) (lam z (app (app add z) (app {s} y))))
    )))");

    app(y(), add_impl)
}

// #[test]
// TODO re-enable
fn add_test() {
    let s = app(app(add(), num(0)), num(1));
    let s = simplify(&s, 30);
    assert_alpha_eq(&s, &num(1));
}
