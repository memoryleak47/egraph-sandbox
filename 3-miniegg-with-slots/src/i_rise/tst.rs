use crate::*;
use crate::i_rise::build::*;

// REDUCTION //

fn reduction_re() -> RecExpr2<RiseENode> {
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

    let add1_re = lam(y, add(var(y), num(1)));
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

#[test]
fn test_reduction() {
    let mut eg = EGraph::new();
    let i = add_rec_expr2(&reduction_re(), &mut eg);
    for _ in 0..30 {
        rewrite_rise(&mut eg);
    }
    let out = extract::<_, AstSizeNoLet>(i.id, &eg);
    assert!(out.node_dag.len() == 16);
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

fn fission_re1() -> RecExpr2<RiseENode> {
    let out = app(symb("map"), fchain(1..=5));
    pattern_to_re(&out)
}

fn fission_re2() -> RecExpr2<RiseENode> {
    let y = 1;

    let left = map1(fchain(3..=5));
    let right = map2(fchain(1..=2), var(y));

    let out = lam(y, app(left, right));

    pattern_to_re(&out)
}

#[test]
fn test_fission() {
    let mut eg = EGraph::new();
    let i1 = add_rec_expr2(&fission_re1(), &mut eg);
    for _ in 0..40 {
        rewrite_rise(&mut eg);
        if let Some(i2) = lookup_rec_expr2(&fission_re2(), &eg) {
            assert_eq!(eg.find_id(i1.id), eg.find_id(i2.id));
            return;
        }
    }

    assert!(false);
}

// BINOMIAL //

fn binomial_re1() -> RecExpr2<RiseENode> {
    todo!()
}

fn binomial_re2() -> RecExpr2<RiseENode> {
    todo!()
}

pub fn test_binomial() {
    let mut eg = EGraph::new();
    let i1 = add_rec_expr2(&binomial_re1(), &mut eg);
    for _ in 0..40 {
        rewrite_rise(&mut eg);
        if let Some(i2) = lookup_rec_expr2(&binomial_re2(), &eg) {
            assert_eq!(eg.find_id(i1.id), eg.find_id(i2.id));
            return;
        }
    }

    assert!(false);
}
