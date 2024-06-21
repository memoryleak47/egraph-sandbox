use crate::*;
use crate::i_rise::build::*;

pub type Problem = (RecExpr<RiseENode>, RecExpr<RiseENode>);

// REDUCTION //

pub fn reduction_re1() -> RecExpr<RiseENode> {
    let comp = 0;
    let add1 = 1;
    let y = 2;
    let f = 3;
    let g = 4;
    let x = 5;

    let comp_re = lam(f,
                    lam(g,
                        lam(x,
                            app(var(f),
                                app(
                                    var(g),
                                    var(x)
                                )
                            )
                        )
                    )
                );

    let add1_re = lam(y, add2(var(y), num(1)));
    let mut it = var(add1);
    for _ in 0..6 {
        it = app(app(var(comp), var(add1)), it);
    }

    let out = app(lam(comp,
            app(lam(add1, it),
                add1_re,
            )
        ),
        comp_re
    );

    pattern_to_re(&out)
}

pub fn reduction_re2() -> RecExpr<RiseENode> {
    let x = 0;
    let mut it = var(x);
    for _ in 0..7 {
        it = add2(it, num(1));
    }

    let out = lam(x, it);

    pattern_to_re(&out)
}

pub fn reduction_problem() -> Problem {
    let start = "(app (lam s0 (app (lam s1 (app (app (var s0) (var s1)) (app (app (var s0) (var s1)) (app (app (var s0) (var s1)) (app (app (var s0) (var s1)) (app (app (var s0) (var s1)) (app (app (var s0) (var s1)) (var s1)))))))) (lam s2 (app (app add (var s2)) 1)))) (lam s4 (lam s5 (lam s3 (app (var s4) (app (var s5) (var s3)))))))".into();
    let goal = "(lam s0 (app (app add (app (app add (app (app add (app (app add (app (app add (app (app add (app (app add (var s0)) 1)) 1)) 1)) 1)) 1)) 1)) 1))".into();

    let start = RecExpr::parse(start).unwrap();
    let goal = RecExpr::parse(goal).unwrap();

    (start, goal)
}

// FISSION //

fn fchain(fs: impl Iterator<Item=usize>) -> Pattern<RiseENode> {
    let x = 42;
    let mut it = var(x);
    for i in fs {
        let f_i = symb(&format!("f{}", i));
        it = app(f_i, it);
    }
    lam(x, it)
}

pub fn fission_re1() -> RecExpr<RiseENode> {
    let out = app(symb("map"), fchain(1..=5));
    pattern_to_re(&out)
}

pub fn fission_re2() -> RecExpr<RiseENode> {
    let y = 1;

    let left = map1(fchain(3..=5));
    let right = map2(fchain(1..=2), var(y));

    let out = lam(y, app(left, right));

    pattern_to_re(&out)
}

pub fn fission_problem() -> Problem {
    (fission_re1(), fission_re2())
}

// BINOMIAL //

pub fn binomial_re1() -> RecExpr<RiseENode> {
    let nbh = 0;
    let dt = dot2(
            join1(symb("weights2d")),
            join1(var(nbh)));
    let out = map2(
        map1(lam(nbh, dt)),
        map2(transpose0(),
            slide3(num(3), num(1), map2(slide2(num(3), num(1)), symb("input")))
        )
    );

    pattern_to_re(&out)
}

pub fn binomial_re2() -> RecExpr<RiseENode> {
    let nbhL = 0;
    let nbhH = 1;
    let nbhV = 2;

    let out = map2(
        lam(nbhL, map2(
            lam(nbhH, dot2(symb("weightsH"), var(nbhH))),
            slide3(num(3), num(1), map2(lam(nbhV, dot2(symb("weightsV"), var(nbhV))), transpose1(var(nbhL))))
        )),
        slide3(num(3), num(1), symb("input"))
    );

    pattern_to_re(&out)
}

pub fn binomial_problem() -> Problem {
    (binomial_re1(), binomial_re2())
}
