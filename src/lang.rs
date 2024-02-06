use std::fmt;

#[derive(Clone, PartialEq, Eq)]
pub enum Term {
    Abstraction(Var, Box<Term>),
    Application(Box<Term>, Box<Term>),
    Var(Var),
}

pub type Var = String;

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Term::Abstraction(v, t) => write!(f, "\\{}. {}", v, t),
            Term::Application(t1, t2) => write!(f, "({} {})", t1, t2),
            Term::Var(v) => write!(f, "{}", v)
        }
    }
}
