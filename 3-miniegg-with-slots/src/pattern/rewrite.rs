use crate::*;

pub struct Rewrite<L: Language> {
    searcher: Pattern<L>,
    applier: Box<dyn Fn(Subst, &mut EGraph<L>)>,
}

pub fn do_rewrites<L: Language>(eg: &mut EGraph<L>, rewrites: &[Rewrite<L>]) {
    let matches: Vec<_> = rewrites.iter().map(|rw| ematch_all(eg, &rw.searcher)).collect();
    for (rw, mtchs) in rewrites.iter().zip(matches.into_iter()) {
        for subst in mtchs {
            (*rw.applier)(subst, eg);
        }
    }
}


// Indirect rewrites.

pub fn mk_rewrite_if<L: Language + 'static>(a: Pattern<L>, b: Pattern<L>, cond: impl Fn(&Subst) -> bool + 'static) -> Rewrite<L> {
    Rewrite {
        searcher: a.clone(),
        applier: Box::new(move |subst, eg| {
            if cond(&subst) {
                let a = pattern_subst(eg, &a, &subst);
                let b = pattern_subst(eg, &b, &subst);
                eg.union(&a, &b);
            }
        }),
    }
}

pub fn mk_rewrite<L: Language + 'static>(a: Pattern<L>, b: Pattern<L>) -> Rewrite<L> {
    mk_rewrite_if(a, b, |_| true)
}

// Direct rewrites.

pub fn rewrite_if<L: Language>(eg: &mut EGraph<L>, a: Pattern<L>, b: Pattern<L>, cond: impl Fn(&Subst) -> bool) {
    for subst in ematch_all(eg, &a) {
        if cond(&subst) {
            let a = pattern_subst(eg, &a, &subst);
            let b = pattern_subst(eg, &b, &subst);
            eg.union(&a, &b);
        }
    }
}

pub fn rewrite<L: Language>(eg: &mut EGraph<L>, a: Pattern<L>, b: Pattern<L>) {
    rewrite_if(eg, a, b, |_| true);
}

pub fn rewrite_bi<L: Language>(eg: &mut EGraph<L>, a: Pattern<L>, b: Pattern<L>) {
    rewrite(eg, a.clone(), b.clone());
    rewrite(eg, b, a);
}

