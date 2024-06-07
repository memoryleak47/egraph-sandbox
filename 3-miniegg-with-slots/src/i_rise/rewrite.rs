use crate::*;
use crate::i_rise::build::*;

// Whether we use extraction-based substitution.
const EXTRACT: bool = false;

fn rules() -> Vec<Rewrite<RiseENode>> {
    let mut rewrites = Vec::new();
    if !EXTRACT {
        rewrites.push(beta());
        rewrites.push(my_let_unused());
        rewrites.push(let_var_same());
        rewrites.push(let_app());
        rewrites.push(let_lam_diff());
    }

    rewrites.push(eta());

    rewrites.push(map_fusion());
    rewrites.push(map_fission());

    rewrites.push(remove_transpose_pair());
    rewrites.push(slide_before_map());
    rewrites.push(map_slide_before_transpose());
    rewrites.push(slide_before_map_map_f());
    rewrites.push(separate_dot_vh_simplified());
    rewrites.push(separate_dot_hv_simplified());
    rewrites
}

pub fn rewrite_rise(eg: &mut EGraph<RiseENode>) {
    // There is no need to compute this in every iteration again.
    let rewrites = rules();

    do_rewrites(eg, &rewrites);

    if EXTRACT {
        beta_extr(eg);
    }
}

fn beta() -> Rewrite<RiseENode> {
    // (\s1. ?b) ?t
    let pat = app(lam(1, pvar("?b")), pvar("?t"));

    // let s1 ?t ?b
    let outpat = let_(1, pvar("?t"), pvar("?b"));

    mk_rewrite(pat, outpat)
}

// extraction-based beta reduction.
fn beta_extr(eg: &mut EGraph<RiseENode>) {
    let pat = app(lam(1, pvar("?b")), pvar("?t"));
    let s = Slot::new(1);

    let extractor = Extractor::<_, AstSize<_>>::new(eg);

    let mut after = Vec::new();
    for subst in ematch_all(eg, &pat) {
        let b = extractor.extract(subst["?b"].clone());
        let t = extractor.extract(subst["?t"].clone());

        let out = re_subst(s, b, &t);
        after.push((subst, out));
    }

    for (subst, out) in after {
        let orig = pattern_subst(eg, &pat, &subst);
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

fn eta() -> Rewrite<RiseENode> {
    // \s1. ?b s1
    let pat = lam(1, app(pvar("?b"), var(1)));

    // ?b
    let outpat = pvar("?b");

    mk_rewrite_if(pat, outpat, |subst| {
        !subst["?b"].slots().contains(&Slot::new(1))
    })
}

fn eta_expansion() -> Rewrite<RiseENode> {
    // ?b
    let pat = pvar("?b");

    // \s1. ?b s1
    let outpat = lam(1, app(pvar("?b"), var(1)));

    mk_rewrite(pat, outpat)
}

fn my_let_unused() -> Rewrite<RiseENode> {
    let pat = let_(1, pvar("?t"), pvar("?b"));
    let outpat = pvar("?b");
    mk_rewrite_if(pat, outpat, |subst| {
        !subst["?b"].slots().contains(&Slot::new(1))
    })
}

fn let_var_same() -> Rewrite<RiseENode> {
    let pat = let_(1, pvar("?e"), var(1));
    let outpat = pvar("?e");
    mk_rewrite(pat, outpat)
}

fn let_app() -> Rewrite<RiseENode> {
    let pat = let_(1, pvar("?e"), app(pvar("?a"), pvar("?b")));
    let outpat = app(
        let_(1, pvar("?e"), pvar("?a")),
        let_(1, pvar("?e"), pvar("?b"))
    );
    mk_rewrite_if(pat, outpat, |subst| {
        subst["?a"].slots().contains(&Slot::new(1)) || subst["?b"].slots().contains(&Slot::new(1))
    })
}

fn let_lam_diff() -> Rewrite<RiseENode> {
    let pat = let_(1, pvar("?e"), lam(2, pvar("?b")));
    let outpat = lam(2,
        let_(1, pvar("?e"), pvar("?b")),
    );
    mk_rewrite_if(pat, outpat, |subst| {
        subst["?b"].slots().contains(&Slot::new(1))
    })
}

fn map_fusion() -> Rewrite<RiseENode> {
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
    mk_rewrite(pat, outpat)
}

fn map_fission() -> Rewrite<RiseENode> {
    let f = || pvar("?f");
    let gx = || pvar("?gx");
    let x = 0;
    let y = 1;

    let pat = map1(lam(x, app(f(), gx())));
    let outpat = lam(y, map2(f(), map2(lam(x, gx()), var(y))));
    mk_rewrite_if(pat, outpat, move |subst| {
        !subst["?f"].slots().contains(&Slot::new(x))
    })
}

fn remove_transpose_pair() -> Rewrite<RiseENode> {
    let pat = transpose1(transpose1(pvar("?x")));
    let outpat = pvar("?x");
    mk_rewrite(pat, outpat)
}

fn slide_before_map() -> Rewrite<RiseENode> {
    let pat = app(
                slide2(pvar("?sz"), pvar("?sp")),
                map2(pvar("?f"), pvar("?y"))
              );

    let outpat = app(
                map1(map1(pvar("?f"))),
                slide3(pvar("?sz"), pvar("?sp"), pvar("?y")),
              );
    mk_rewrite(pat, outpat)
}

fn map_slide_before_transpose() -> Rewrite<RiseENode> {
    let pat = transpose1(map2(
        slide2(pvar("?sz"), pvar("?sp")),
        pvar("?y")
    ));
    let outpat = map2(transpose0(),
        slide3(pvar("?sz"), pvar("?sp"), transpose1(pvar("?y")))
    );
    mk_rewrite(pat, outpat)
}

fn slide_before_map_map_f() -> Rewrite<RiseENode> {
    let pat = map2(map1(pvar("?f")),
        slide3(pvar("?sz"), pvar("?sp"), pvar("?y"))
    );
    let outpat = slide3(pvar("?sz"), pvar("?sp"),
        map2(pvar("?f"), pvar("?y"))
    );
    mk_rewrite(pat, outpat)
}

fn separate_dot_vh_simplified() -> Rewrite<RiseENode> {
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
    mk_rewrite(pat, outpat)
}

fn separate_dot_hv_simplified() -> Rewrite<RiseENode> {
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
    mk_rewrite(pat, outpat)
}
