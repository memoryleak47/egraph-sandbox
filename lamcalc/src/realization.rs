use crate::*;

pub trait Realization: Sized {
    fn to_ast_string(&self) -> String;
    fn from_ast(ast: &Ast) -> Self;
    fn simplify(&self, steps: u32) -> Self;
    fn find_eq(&self, other: &Self, steps: u32) -> bool;

    fn to_ast(&self) -> Ast {
        Ast::parse(&self.to_ast_string())
    }

    fn from_ast_string(s: &str) -> Self {
        Self::from_ast(&Ast::parse(s))
    }
}

pub fn simplify<R: Realization>(s: &str, steps: u32) -> String {
    let s = R::from_ast_string(s);
    let s = s.simplify(steps);
    let s = s.to_ast_string();
    s
}

pub fn check_simplify<R: Realization>(p: &str, steps: u32) {
    let out1 = simplify::<R>(p, steps);
    let out2 = run(p);
    assert_alpha_eq(&*out1, &*out2);
}

// checks whether simplify has valid output, even though it might not be able to finish the whole computation.
pub fn check_simplify_incomplete<R: Realization>(p: &str, steps: u32) {
    let out1 = run(&simplify::<R>(p, steps));
    let out2 = run(p);
    assert_alpha_eq(&*out1, &*out2);
}

pub fn check_eq<R: Realization>(s1: &str, s2: &str, steps: u32) {
    let s1 = R::from_ast_string(s1);
    let s2 = R::from_ast_string(s2);
    assert!(R::find_eq(&s1, &s2, steps));
}

// Non-Realization functions:

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
