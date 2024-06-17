use crate::*;
use crate::i_rise::build::*;

pub enum SubstMethod {
    Extraction,
    SmallStep,
}

pub fn rise_rules(subst_m: SubstMethod) -> Vec<Rewrite<RiseENode>> {
    let mut rewrites = Vec::new();

    rewrites.push(eta());

    rewrites.push(map_fusion());
    rewrites.push(map_fission());

    rewrites.push(remove_transpose_pair());
    rewrites.push(slide_before_map());
    rewrites.push(map_slide_before_transpose());
    rewrites.push(slide_before_map_map_f());
    rewrites.push(separate_dot_vh_simplified());
    rewrites.push(separate_dot_hv_simplified());

    match subst_m {
        SubstMethod::Extraction => {
            rewrites.push(beta_extr_direct());
        },
        SubstMethod::SmallStep => {
            rewrites.push(beta());
            rewrites.push(my_let_unused());
            rewrites.push(let_var_same());
            rewrites.push(let_app());
            rewrites.push(let_lam_diff());
        },
    }

    rewrites
}

fn beta() -> Rewrite<RiseENode> {
    let pat = Pattern::parse("(app (lam s1 ?body) ?e)").unwrap();
    let outpat = Pattern::parse("(let s1 ?e ?body)").unwrap();

    mk_rewrite(pat, outpat)
}

fn eta() -> Rewrite<RiseENode> {
    let pat = Pattern::parse("(lam s1 (app ?f (var s1)))").unwrap();
    let outpat = Pattern::parse("?f").unwrap();

    mk_rewrite_if(pat, outpat, |subst| {
        !subst["f"].slots().contains(&Slot::new(1))
    })
}

fn eta_expansion() -> Rewrite<RiseENode> {
    let pat = Pattern::parse("?f").unwrap();
    let outpat = Pattern::parse("(lam s1 (app ?f (var s1)))").unwrap();

    mk_rewrite(pat, outpat)
}

fn my_let_unused() -> Rewrite<RiseENode> {
    let pat = Pattern::parse("(let s1 ?t ?b)").unwrap();
    let outpat = Pattern::parse("?b").unwrap();
    mk_rewrite_if(pat, outpat, |subst| {
        !subst["b"].slots().contains(&Slot::new(1))
    })
}

fn let_var_same() -> Rewrite<RiseENode> {
    let pat = Pattern::parse("(let s1 ?e (var s1))").unwrap();
    let outpat = Pattern::parse("?e").unwrap();
    mk_rewrite(pat, outpat)
}

fn let_app() -> Rewrite<RiseENode> {
    let pat = Pattern::parse("(let s1 ?e (app ?a ?b))").unwrap();
    let outpat = Pattern::parse("(app (let s1 ?e ?a) (let s1 ?e ?b))").unwrap();
    mk_rewrite_if(pat, outpat, |subst| {
        subst["a"].slots().contains(&Slot::new(1)) || subst["b"].slots().contains(&Slot::new(1))
    })
}

fn let_lam_diff() -> Rewrite<RiseENode> {
    let pat = Pattern::parse("(let s1 ?e (lam s2 ?body))").unwrap();
    let outpat = Pattern::parse("(lam s2 (let s1 ?e ?body))").unwrap();
    mk_rewrite_if(pat, outpat, |subst| {
        subst["body"].slots().contains(&Slot::new(1))
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

// subst using extraction
fn beta_extr() -> Rewrite<RiseENode> {
    let pat = app(lam(1, pvar("?b")), pvar("?t"));
    let s = Slot::new(1);

    let a = pat.clone();
    let a2 = pat.clone();

    let rt: RewriteT<RiseENode, Vec<(Subst, RecExpr<RiseENode>)>> = RewriteT {
        searcher: Box::new(move |eg| {
            let extractor = Extractor::<_, AstSize>::new(eg);

            let mut out: Vec<(Subst, RecExpr<RiseENode>)> = Vec::new();
            for subst in ematch_all(eg, &a) {
                let b = extractor.extract(subst["?b"].clone(), eg);
                let t = extractor.extract(subst["?t"].clone(), eg);
                let res = re_subst(s, b, &t);
                out.push((subst, res));
            }
            out
        }),
        applier: Box::new(move |substs, eg| {
            for (subst, res) in substs {
                let orig = pattern_subst(eg, &pat, &subst);
                let res = eg.add_expr(res);
                eg.union(&orig, &res);
            }
        }),
    };
    rt.into()
}

// why is this faster than beta_extr?
// Probably because it can extract smaller terms after more rewrites?
fn beta_extr_direct() -> Rewrite<RiseENode> {
    let pat = app(lam(1, pvar("?b")), pvar("?t"));
    let s = Slot::new(1);

    let a = pat.clone();
    let a2 = pat.clone();

    let rt: RewriteT<RiseENode, ()> = RewriteT {
        searcher: Box::new(|_| ()),
        applier: Box::new(move |(), eg| {
            let extractor = Extractor::<_, AstSize>::new(eg);

            let mut out: Vec<(Subst, RecExpr<RiseENode>)> = Vec::new();
            for subst in ematch_all(eg, &a) {
                let b = extractor.extract(subst["?b"].clone(), eg);
                let t = extractor.extract(subst["?t"].clone(), eg);
                let res = re_subst(s, b, &t);
                out.push((subst, res));
            }
            for (subst, res) in out {
                let orig = pattern_subst(eg, &pat, &subst);
                let res = eg.add_expr(res);
                eg.union(&orig, &res);
            }
        }),
    };
    rt.into()
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
