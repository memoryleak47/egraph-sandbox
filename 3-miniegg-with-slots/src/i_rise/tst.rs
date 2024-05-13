use crate::*;

// REDUCTION //

fn reduction_re() -> RecExpr2<RiseENode> {
    let comp = 0;
    let add1 = 1;
    let y = 2;
    let f = 3;
    let g = 4;
    let x = 5;

    let comp_re = lam_re(f,
                    lam_re(g,
                        lam_re(x,
                            app_re(var_re(f),
                                app_re(
                                    var_re(g),
                                    var_re(x)
                                )
                            )
                        )
                    )
                );

    let add1_re = lam_re(y, add_re(var_re(y), num_re(1)));
    let mut it = var_re(add1);
    for _ in 0..6 {
        it = app_re(app_re(var_re(comp), var_re(add1)), it);
    }

    app_re(lam_re(comp,
            app_re(lam_re(add1, it),
                add1_re,
            )
        ),
        comp_re
    )
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

fn fchain(fs: impl Iterator<Item=usize>) -> RecExpr2<RiseENode> {
    let x = 42;
    let mut it = var_re(x);
    for i in fs {
        let f_i = symb_re(&format!("f{}", i));
        it = app_re(f_i, it);
    }
    lam_re(x, it)
}

fn fission_re1() -> RecExpr2<RiseENode> {
    app_re(symb_re("map"), fchain(1..=5))
}

fn fission_re2() -> RecExpr2<RiseENode> {
    let map = || symb_re("map");
    let y = 1;

    let left = app_re(map(), fchain(3..=5));
    let right = app_re(app_re(map(), fchain(1..=2)), var_re(y));

    lam_re(y, app_re(left, right))
}

// #[test]
pub fn test_fission() {
    let mut eg = EGraph::new();
    let i1 = add_rec_expr2(&fission_re1(), &mut eg);
    for _ in 0..30 {
        rewrite_rise(&mut eg);
    }
    let i2 = lookup_rec_expr2(&fission_re2(), &eg).unwrap();
    assert_eq!(i1, i2);
}
