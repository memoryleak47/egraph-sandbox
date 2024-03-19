use crate::*;

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
];

struct TestCase {
    input: &'static str,
    output: &'static str,
    steps: usize,
}

#[test]
fn test_egraph_roundtrip() {
    for t in TEST_CASES {
        let re = parse(t.input);
        let mut eg = EGraph::new();
        let i = eg.add_expr(re.clone());
        let re = extract(i, &eg);
        let s = to_string(re);
        assert_eq!(t.input, s);
    }
}

#[test]
fn test_beta_reduction() {
    for t in TEST_CASES {
        let re = parse(t.input);
        let mut eg = EGraph::new();
        let i = eg.add_expr(re.clone());

        for _ in 0..t.steps {
            rewrite_step(&mut eg);
        }

        let re = extract(i, &eg);
        let s = to_string(re);
        assert_eq!(t.output, s);
    }
}
