use crate::*;

pub trait Realization: Sized {
    type Id: Clone + Eq;

    fn new() -> Self { todo!() }
    fn add_ast(&mut self, ast: &Ast) -> Self::Id { todo!() }
    fn extract_ast(&self, id: Self::Id) -> Ast { todo!() }
    fn find(&self, id: Self::Id) -> Self::Id { todo!() }
    fn step(&mut self) { todo!() }
    fn enode_count(&self) -> usize { todo!() }
    fn eclass_count(&self) -> usize { todo!() }
}

pub fn simplify<R: Realization>(s: &str) -> String {
    let ast = Ast::parse(s);
    let mut eg = R::new();
    let i = eg.add_ast(&ast);
    for _ in 0..200 {
        eg.step();
        if eg.enode_count() > 40000 {
            break;
        }
    }
    let out = eg.extract_ast(i.clone());
    let out = out.to_string();

    out
}

// TODO the smallest term isn't necessarily the beta-NF.
pub fn check_simplify<R: Realization>(p: &str) {
    let out1 = simplify::<R>(p);
    let out2 = run(p);
    assert_alpha_eq(&*out1, &*out2);
}

// checks whether simplify has valid output, even though it might not be able to finish the whole computation.
pub fn check_simplify_incomplete<R: Realization>(p: &str) {
    let out1 = run(&simplify::<R>(p));
    let out2 = run(p);
    assert_alpha_eq(&*out1, &*out2);
}

pub fn check_eq<R: Realization>(s1: &str, s2: &str, steps: u32) {
    let s1 = Ast::parse(s1);
    let s2 = Ast::parse(s2);
    let mut eg = R::new();
    let i1 = eg.add_ast(&s1);
    let i2 = eg.add_ast(&s2);
    for _ in 0..200 {
        eg.step();
        if eg.enode_count() > 40000 {
            break;
        }
    }
    assert!(eg.find(i1) == eg.find(i2));
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
