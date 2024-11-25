use crate::*;

define_language! {
pub enum Rise {
    // lambda calculus:
    Lam(Bind<AppliedId>) = "lam",
    App(AppliedId, AppliedId) = "app",
    Var(Slot) = "var",
    Let(Bind<AppliedId>, AppliedId) = "let",

    // rest:
    Number(u32),
    Symbol(Symbol),
}
}
