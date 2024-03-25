use crate::*;

#[macro_export]
macro_rules! unpack_tests {
    ($R:ident) => {
        #[test]
        fn test_cannot_simplify() {
            let s = [
                "(lam x0 x0)",
                "(lam x0 (lam x1 x0))",
                "(lam x0 (lam x1 x1))",
                "(lam x0 (lam x1 (app x0 x1)))",
            ];

            for p in s {
                let out = simplify::<$R>(p, 10);
                assert_alpha_eq(&*out, p);
            }
        }

        #[test]
        fn test_self_rec() {
            // The intereting thing about this test is the following:
            // "\x. (\y. y) x -> \x. x" using beta reduction.
            //
            // and "\x. x -> \y. y" by alpha conversion.
            //
            // Thus, we suddenly have a self-recursive EClass, for Realizations that share across alpha-equivalence.
            // C = \y. y | \z. C z
            //
            // This sometimes causes infinite loops, if you iterate by depth-first-search.
            let s = "(lam x (app (lam y y) x))";
            check_simplify::<$R>(&s, 2);
        }

        #[test]
        fn test_nested_identity1() {
            let p = "(app (lam x0 x0) (lam x1 x1))";
            check_simplify::<$R>(p, 10);
        }

        #[test]
        fn test_nested_identity2() {
            let p = "(app (lam x0 x0) (lam x1 (app x1 x1)))";
            check_simplify::<$R>(p, 10);
        }

        #[test]
        fn test_nested_identity3() {
            let p = "(app (lam x0 (app x0 x0)) (lam x1 x1))";
            check_simplify::<$R>(p, 10);
        }

        #[test]
        fn test_simple_beta() {
            let p = "(lam x (lam y
                (app
                    (lam z (app x z))
                y)
            ))";
            check_simplify::<$R>(p, 10);
        }

        #[test]
        fn test_redudant_slot() {
            // y is unused, and hence x is effectively redundant.
            let p = "(lam x (app (lam y (lam z z)) x))";
            check_simplify::<$R>(p, 10);
        }

        #[test]
        fn test_add() {
            let s = app(app(add(), num(0)), num(1));
            check_simplify::<$R>(&s, 5);
        }

        #[test]
        fn test_add_incomplete() {
            let s = app(app(add(), num(2)), num(3));
            check_simplify_incomplete::<$R>(&s, 5);
        }

        #[test]
        fn test_inf_loop() {
            let p = "(app (lam x0 (app x0 x0)) (lam x1 (app x1 x1)))";
            let out = simplify::<$R>(p, 3);
            assert_alpha_eq(&out, p);
        }

        // A y-combinator example that directly yields "f x = x" without looping.
        #[test]
        fn test_y_identity() {
            let p = "(lam f (lam arg arg))";
            let s = app(y(), String::from(p));

            let out = simplify::<$R>(&s, 30);
            assert_alpha_eq(&out, "(lam x x)");
        }

        #[test]
        fn test_add_y_step() {
            let s1 = app(add_impl(), add());
            let s2 = add();
            check_eq::<$R>(&s1, &s2, 1);
        }
    }
}
