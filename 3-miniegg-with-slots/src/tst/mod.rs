use crate::*;

mod y;

static TEST_CASES: &'static [TestCase] = &[
    //// identity test cases ////
    TestCase {
        input: "(lam x0 x0)",
        output: "(lam x0 x0)",
        steps: 0,
    },

    TestCase {
        input: "(lam x0 (lam x1 x0))",
        output: "(lam x0 (lam x1 x0))",
        steps: 0,
    },

    TestCase {
        input: "(lam x0 (lam x1 x1))",
        output: "(lam x0 (lam x1 x1))",
        steps: 0,
    },

    TestCase {
        input: "(lam x0 (lam x1 (app x0 x1)))",
        output: "(lam x0 (lam x1 (app x0 x1)))",
        steps: 0,
    },

    TestCase {
        input: "(app (lam x0 (app x0 x0)) (lam x1 (app x1 x1)))",
        output: "(app (lam x0 (app x0 x0)) (lam x1 (app x1 x1)))",
        steps: 0,
    },

    //// Beta-Reduction test cases ////

    TestCase {
        input: "(app (lam x0 x0) (lam x1 x1))",
        output: "(lam x0 x0)",
        steps: 1,
    },
    TestCase {
        input: "(app (lam x0 (app x0 x0)) (lam x1 x1))",
        output: "(lam x0 x0)",
        steps: 2,
    },
    TestCase { // The infinite loop!
        input: "(app (lam x0 (app x0 x0)) (lam x1 (app x1 x1)))",
        output: "(app (lam x0 (app x0 x0)) (lam x1 (app x1 x1)))",
        steps: 10,
    },
    TestCase {
        input: "(lam x0 (lam x1 (app (lam x2 (app x0 x2)) x1)))",
        output: "(lam x0 (lam x1 (app x0 x1)))",
        steps: 1,
    },
    TestCase { // It looks as if x0 is used, but it's not. This is an example of a redundant slot.
        input: "(lam x0 (app (lam x1 (lam x2 x2)) x0))",
        output: "(lam x0 (lam x1 x1))",
        steps: 1,
    },
];

struct TestCase {
    input: &'static str,
    output: &'static str,
    steps: usize,
}

fn roundtrip(input: &str) -> String {
    let re = parse(input);
    let mut eg = EGraph::new();
    let i = eg.add_expr(re.clone());
    let re = extract(i, &eg);

    to_string(re)
}

fn simplify(input: &str, steps: usize) -> String {
    let re = parse(input);
    let mut eg = EGraph::new();
    let i = eg.add_expr(re.clone());

    for _ in 0..steps {
        rewrite_step(&mut eg);
    }

    let re = extract(i, &eg);

    to_string(re)
}

// TODO it would be better to have an alpha-equivalence test that doesn't depend on the EGraph. Otherwise it's not great to test the EGraph!
fn assert_alpha_eq(l: &str, r: &str) {
    assert_eq!(roundtrip(l), roundtrip(r));
}

#[test]
fn test_egraph_roundtrip() {
    for t in TEST_CASES {
        let out = roundtrip(t.input);
        assert_eq!(&*out, t.input);
    }
}

#[test]
fn test_beta_reduction() {
    for t in TEST_CASES {
        let out = simplify(t.input, t.steps);
        assert_eq!(&*out, t.output);
    }
}

#[test]
fn alpha_eq() {
    let x = "(lam x (app (lam x x) x))";
    let y = "(lam y (app (lam z z) y))";

    assert_alpha_eq(x, y);
}
