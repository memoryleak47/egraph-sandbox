use crate::*;

// The Explanation data type.

#[derive(Clone)]
pub struct Explanation<L: Language> {
    pub term: RecExpr<L>,
    pub step: Option<Box<ExplanationStep<L>>>,
}

impl<L: Language> Explanation<L> {
    pub fn last(&self) -> &RecExpr<L> {
        if let Some(step) = &self.step {
            step.exp.last()
        } else {
            &self.term
        }
    }
}

#[derive(Clone)]
pub struct ExplanationStep<L: Language> {
    pub index_list: Vec<usize>,
    pub justification: Justification, // TODO is_forward is missing!
    pub exp: Explanation<L>,
}

// panics if a.last_term != b.first_term
pub fn compose_explanation<L: Language>(a: Explanation<L>, b: Explanation<L>) -> Explanation<L> {
    if a.step.is_none() {
        assert!(alpha_eq(&a.term, &b.term));
        return b;
    }

    let mut a = a;
    let mut r: &mut ExplanationStep<L> = a.step.as_mut().unwrap();
    loop {
        if r.exp.step.is_some() {
            let Some(step) = r.exp.step.as_mut() else { panic!() };
            r = step;
        } else {
            assert!(alpha_eq(&r.exp.term, &b.term));
            r.exp = b;
            return a;
        }
    }
}

pub fn compose_explanation_list<L: Language>(l: Vec<Explanation<L>>) -> Explanation<L> {
    let mut l = l;
    let mut out = l.pop().unwrap();

    for x in l.into_iter().rev() {
        out = compose_explanation(x, out);
    }

    out
}


