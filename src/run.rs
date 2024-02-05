use crate::lang::*;


pub fn run(mut t: Term) -> Term {
    loop {
        match step(t) {
            (x, true) => t = x,
            (x, false) => return x,
        }
    }
}

// tries to do one step, will return `true` if it was successful.
fn step(t: Term) -> (Term, bool) {
    match t {
        Term::Application(box Term::Abstraction(v, box e1), box e2) => {
            (substitute(e1, &v, &e2), true)
        },
        Term::Abstraction(v, box e) => {
            let (e, success) = step(e);
            (Term::Abstraction(v, Box::new(e)), success)
        },
        Term::Application(box e1, box e2) => {
            let (e1, success) = step(e1);
            if success { return (Term::Application(Box::new(e1), Box::new(e2)), true); }
            let (e2, success) = step(e2);
            (Term::Application(Box::new(e1), Box::new(e2)), success)
        },
        Term::Var(v) => (Term::Var(v), false),
    }
}

// t[e/v]
fn substitute(t: Term, v: &Var, e: &Term) -> Term {
    match t {
        Term::Abstraction(v2, box e1) if *v == v2 => Term::Abstraction(v2, Box::new(e1)),
        Term::Abstraction(v2, box e1) => {
            let e1 = substitute(e1, v, e);
            Term::Abstraction(v2, Box::new(e1))
        },
        Term::Application(box e1, box e2) => {
            let e1 = substitute(e1, v, e);
            let e2 = substitute(e2, v, e);
            Term::Application(Box::new(e1), Box::new(e2))
        },
        Term::Var(v2) if *v == v2 => e.clone(),
        Term::Var(v2) => Term::Var(v2),
    }
}
