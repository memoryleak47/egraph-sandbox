use crate::*;

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
