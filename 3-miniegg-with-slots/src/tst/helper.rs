use crate::*;

pub fn simplify(input: &str, steps: u32) -> String {
    let input = &norm(input);

    let re = RecExpr::parse(input);
    let mut eg = EGraph::new();
    let i = eg.add_expr(re.clone());

    eg.inv();
    for _ in 0..steps {
        rewrite_step(&mut eg);
        eg.inv();
    }

    let re = extract(i, &eg);

    re.to_string()
}

pub fn norm(s: &str) -> String {
    Ast::parse(s).normalize().to_string()
}

pub fn run(s: &str) -> String {
    Ast::parse(s).run().normalize().to_string()
}

pub fn assert_alpha_eq(s1: &str, s2: &str) {
    assert_eq!(norm(s1), norm(s2));
}

pub fn assert_run_eq(s1: &str, s2: &str) {
    assert_eq!(run(s1), run(s2));
}

pub fn check_simplify(p: &str, steps: u32) {
    let out1 = simplify(p, steps);
    let out2 = run(p);
    assert_alpha_eq(&*out1, &*out2);
}
