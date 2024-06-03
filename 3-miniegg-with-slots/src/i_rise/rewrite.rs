use crate::*;
use crate::i_rise::build::*;

pub fn rewrite_rise(eg: &mut EGraph<RiseENode>) {
    beta(eg);
    // beta_extr(eg);
    eta(eg);
    // eta_expansion(eg);

    my_let_unused(eg);
    let_var_same(eg);
    let_app(eg);
    let_lam_diff(eg);

    map_fusion(eg);
    map_fission(eg);

    remove_transpose_pair(eg);
    slide_before_map(eg);
    map_slide_before_transpose(eg);
    slide_before_map_map_f(eg);
    separate_dot_vh_simplified(eg);
    separate_dot_hv_simplified(eg);
}

fn beta(eg: &mut EGraph<RiseENode>) {
    // (\s1. ?b) ?t
    let pat = app(lam(1, pvar("?b")), pvar("?t"));

    // let s1 ?t ?b
    let outpat = let_(1, pvar("?t"), pvar("?b"));

    rewrite(eg, pat, outpat);
}

// extraction-based beta reduction.
fn beta_extr(eg: &mut EGraph<RiseENode>) {
    let pat = app(lam(1, pvar("?b")), pvar("?t"));
    let s = Slot::new(1);
    for subst in ematch_all(eg, &pat) {
        let orig = pattern_subst(eg, &pat, &subst);

        let b = ast_size_extract(subst["?b"].clone(), eg);
        let t = ast_size_extract(subst["?t"].clone(), eg);

        let out = re_subst(s, b, &t);
        let out = eg.add_expr(out);

        eg.union(&orig, &out);
    }
}

fn re_subst(s: Slot, b: RecExpr<RiseENode>, t: &RecExpr<RiseENode>) -> RecExpr<RiseENode> {
    let new_node = match b.node {
        RiseENode::Var(s2) if s == s2 => return t.clone(),
        RiseENode::Lam(s2, _) if s == s2 => panic!("This shouldn't be possible!"),
        RiseENode::Let(..) => panic!("This shouldn't be here!"),
        old => old,
    };

    let mut children = Vec::new();
    for child in b.children {
        children.push(re_subst(s, child, t));
    }

    RecExpr {
        node: new_node,
        children,
    }
}

fn eta(eg: &mut EGraph<RiseENode>) {
    // \s1. ?b s1
    let pat = lam(1, app(pvar("?b"), var(1)));

    // ?b
    let outpat = pvar("?b");

    rewrite_if(eg, pat, outpat, |subst| {
        !subst["?b"].slots().contains(&Slot::new(1))
    });
}

fn eta_expansion(eg: &mut EGraph<RiseENode>) {
    // ?b
    let pat = pvar("?b");

    // \s1. ?b s1
    let outpat = lam(1, app(pvar("?b"), var(1)));

    rewrite(eg, pat, outpat);
}

fn my_let_unused(eg: &mut EGraph<RiseENode>) {
    let pat = let_(1, pvar("?t"), pvar("?b"));
    let outpat = pvar("?b");
    rewrite_if(eg, pat, outpat, |subst| {
        !subst["?b"].slots().contains(&Slot::new(1))
    });
}

fn let_var_same(eg: &mut EGraph<RiseENode>) {
    let pat = let_(1, pvar("?e"), var(1));
    let outpat = pvar("?e");
    rewrite(eg, pat, outpat);
}

fn let_app(eg: &mut EGraph<RiseENode>) {
    let pat = let_(1, pvar("?e"), app(pvar("?a"), pvar("?b")));
    let outpat = app(
        let_(1, pvar("?e"), pvar("?a")),
        let_(1, pvar("?e"), pvar("?b"))
    );
    rewrite_if(eg, pat, outpat, |subst| {
        subst["?a"].slots().contains(&Slot::new(1)) || subst["?b"].slots().contains(&Slot::new(1))
    });
}

fn let_lam_diff(eg: &mut EGraph<RiseENode>) {
    let pat = let_(1, pvar("?e"), lam(2, pvar("?b")));
    let outpat = lam(2,
        let_(1, pvar("?e"), pvar("?b")),
    );
    rewrite_if(eg, pat, outpat, |subst| {
        subst["?b"].slots().contains(&Slot::new(1))
    });
}

fn map_fusion(eg: &mut EGraph<RiseENode>) {
    let f = || pvar("?f");
    let g = || pvar("?g");
    let arg = || pvar("?arg");
    let x = 0;
    let pat = map2(f(),
                map2(g(), arg())
              );
    let outpat = map2(
            lam(x, app(f(), app(g(), var(x)))),
        arg());
    rewrite(eg, pat, outpat);
}

fn map_fission(eg: &mut EGraph<RiseENode>) {
    let f = || pvar("?f");
    let gx = || pvar("?gx");
    let x = 0;
    let y = 1;

    let pat = map1(lam(x, app(f(), gx())));
    let outpat = lam(y, map2(f(), map2(lam(x, gx()), var(y))));
    rewrite_if(eg, pat, outpat, |subst| {
        !subst["?f"].slots().contains(&Slot::new(x))
    });
}

fn remove_transpose_pair(eg: &mut EGraph<RiseENode>) {
    let pat = transpose1(transpose1(pvar("?x")));
    let outpat = pvar("?x");
    rewrite(eg, pat, outpat);
}

fn slide_before_map(eg: &mut EGraph<RiseENode>) {
    let pat = app(
                slide2(pvar("?sz"), pvar("?sp")),
                map2(pvar("?f"), pvar("?y"))
              );

    let outpat = app(
                map1(map1(pvar("?f"))),
                slide3(pvar("?sz"), pvar("?sp"), pvar("?y")),
              );
    rewrite(eg, pat, outpat);
}

fn map_slide_before_transpose(eg: &mut EGraph<RiseENode>) {
    let pat = transpose1(map2(
        slide2(pvar("?sz"), pvar("?sp")),
        pvar("?y")
    ));
    let outpat = map2(transpose0(),
        slide3(pvar("?sz"), pvar("?sp"), transpose1(pvar("?y")))
    );
    rewrite(eg, pat, outpat);
}

fn slide_before_map_map_f(eg: &mut EGraph<RiseENode>) {
    let pat = map2(map1(pvar("?f")),
        slide3(pvar("?sz"), pvar("?sp"), pvar("?y"))
    );
    let outpat = slide3(pvar("?sz"), pvar("?sp"),
        map2(pvar("?f"), pvar("?y"))
    );
    rewrite(eg, pat, outpat);
}

fn separate_dot_vh_simplified(eg: &mut EGraph<RiseENode>) {
    let x = 0;
    let sdvh = 1;

    let pat = reduce3(add0(), num(0),
        map2(
            lam(x, mul2(fst1(var(x)), snd1(var(x)))),
            zip2(join1(symb("weights2d")), join1(pvar("?nbh"))),
        ),
    );
    let outpat = reduce3(add0(), num(0),
        map2(
            lam(x, mul2(fst1(var(x)), snd1(var(x)))),
            zip2(symb("weightsH"),
                map2(
                    lam(sdvh,
                        reduce3(add0(), num(0),
                            map2(
                                lam(x, mul2(fst1(var(x)), snd1(var(x)))),
                                zip2(symb("weightsV"), var(sdvh))
                            ),
                        ),
                    ),
                    transpose1(pvar("?nbh")),
                ),
            ),
        ),
    );
    rewrite(eg, pat, outpat);
}

fn separate_dot_hv_simplified(eg: &mut EGraph<RiseENode>) {
    let x = 0;
    let sdhv = 1;

    let pat = reduce3(add0(), num(0),
        map2(
            lam(x, mul2(fst1(var(x)), snd1(var(x)))),
            zip2(join1(symb("weights2d")), join1(pvar("?nbh"))),
        ),
    );
    let outpat = reduce3(add0(), num(0),
        map2(
            lam(x, mul2(fst1(var(x)), snd1(var(x)))),
            zip2(symb("weightsV"),
                map2(
                    lam(sdhv,
                        reduce3(add0(), num(0),
                            map2(
                                lam(x, mul2(fst1(var(x)), snd1(var(x)))),
                                zip2(symb("weightsH"), var(sdhv))
                            ),
                        ),
                    ),
                    pvar("?nbh"),
                ),
            ),
        ),
    );
    rewrite(eg, pat, outpat);
}
