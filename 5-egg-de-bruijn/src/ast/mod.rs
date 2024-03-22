use crate::*;

// NOTE: This is a copy of the AST implementation from 3-miniegg-with-slots.
// Changes shouldn't be done in here!
// This should be equivalent to 3-miniegg-with-slots/src/ast!

mod parse;
mod display;
mod step;
mod normalize;

#[derive(Clone)]
pub enum Ast {
    Lam(String, Box<Ast>),
    App(Box<Ast>, Box<Ast>),
    Var(String),
}
