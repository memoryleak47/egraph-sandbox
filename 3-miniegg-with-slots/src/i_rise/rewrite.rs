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
            rewrites.push(beta_extr_preserving());
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
    // (\s1. ?b) ?t
    let pat = app(lam(1, pvar("?b")), pvar("?t"));

    // let s1 ?t ?b
    let outpat = let_(1, pvar("?t"), pvar("?b"));

    mk_rewrite(pat, outpat)
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

// TODO why is this faster than beta_extr?
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

type Ext = Extractor<RiseENode, AstSize>;

use std::sync::Mutex;
lazy_static::lazy_static! {
    // I have sinned. But this is a hack anyways.
    static ref EXT: Mutex<Ext> = Mutex::new(Ext { map: Default::default() });
}

// It's the same rule as beta_extr, but it remembers its Extractor from the previous iteration.
// It will keep the same choices from the iteration before, as long as it's possible.
// This shrinks the extraction coverage and yields a smaller egraph. (at least in theory)
fn beta_extr_preserving() -> Rewrite<RiseENode> {
    let pat = app(lam(1, pvar("?b")), pvar("?t"));
    let s = Slot::new(1);

    let a = pat.clone();
    let a2 = pat.clone();

    let rt: RewriteT<RiseENode, Vec<(Subst, RecExpr<RiseENode>)>> = RewriteT {
        searcher: Box::new(move |eg| {
            let mut guard = EXT.lock().unwrap();
            let extractor = Extractor::<_, AstSize>::new(eg);
            let extractor = merge_extractors(extractor, &*guard, eg);

            let mut out: Vec<(Subst, RecExpr<RiseENode>)> = Vec::new();
            for subst in ematch_all(eg, &a) {
                let b = extractor.extract(subst["?b"].clone(), eg);
                let t = extractor.extract(subst["?t"].clone(), eg);
                let res = re_subst(s, b, &t);
                out.push((subst, res));
            }

            *guard = extractor;

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

// All shapes from `old` that are still optimal in `new` should be used in it.
fn merge_extractors(mut new: Ext, old: &Ext, eg: &EGraph<RiseENode>) -> Ext {
    for (_, WithOrdRev(enode, _)) in &old.map {
        let enode = eg.find_enode(enode);
        if let Some(i) = eg.lookup(&enode) {
            // converts enode to "normal-form".
            let enode = eg.enodes(i.id).into_iter().find(|x| x.shape().0 == enode.shape().0).unwrap();

            let c1: u64 = AstSize::cost(&enode, |i| new.map[&i].1);
            let WithOrdRev(enode2, c2) = &new.map[&i.id];
            if c1 == *c2 {
                new.map.insert(i.id, WithOrdRev(enode, *c2));
            }
        }
    }
    new
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
