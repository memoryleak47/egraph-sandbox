use crate::*;

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

    app_re(
        app_re(
            lam_re(comp,
                lam_re(add1, it)
            ),
            add1_re,
        ),
        comp_re
    )
}

// #[test]
pub fn rise_test_reduction() {
    let mut eg = EGraph::new();
    let i = add_rec_expr2(&reduction_re(), &mut eg);
    for _ in 0..200 {
        rewrite_rise(&mut eg);
    }
    let out = extract::<_, AstSizeNoLet>(i.id, &eg);
    dbg!(&out);
}
