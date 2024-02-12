use crate::*;

define_language! {
    pub enum Term {
        "lam" = Abstraction([Id; 2]), // TODO the left arg of `lam` should only be a variable, not a full-blown Term.
        "app" = Application([Id; 2]),
        Symb(Symbol),

        "+" = Add([Id; 2]),
        "*" = Mul([Id; 2]),
        Num(i32),
    }
}
