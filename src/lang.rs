#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Term {
    Abstraction(Var, Box<Term>),
    Application(Box<Term>, Box<Term>),
    Var(Var),
}

pub type Var = String;
