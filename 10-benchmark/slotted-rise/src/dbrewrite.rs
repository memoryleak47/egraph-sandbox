use crate::*;

/*
pub enum RiseSubstMethod {
    Extraction,
    SmallStep, // <<< only this implemented
    SmallStepUnoptimized,
}
*/

pub fn db_rise_rules(subst_m: RiseSubstMethod) -> Vec<Rewrite<Rise>> {
    let mut rewrites = Vec::new();

    rewrites.push(eta());
    // rewrites.push(eta_expansion());

    rewrites.push(map_fusion());
    rewrites.push(map_fission());

    rewrites.push(beta());

    rewrites.push(sig_unused());
    rewrites.push(phi_unused());
    rewrites.push(sig_lam());
    rewrites.push(phi_lam());
    rewrites.push(sig_app());
    rewrites.push(phi_app());
    rewrites.push(sig_var_cons());
    rewrites.push(phi_var_const());

    rewrites
}

fn beta() -> Rewrite<Rise> {
    Rewrite::new("beta",
      "(app (lam ?body) ?e)",
      "(sig 0 ?body ?e)")
}

fn eta() -> Rewrite<Rise> {
    Rewrite::new_if("eta",
      "(lam (app ?f %0))",
      "(phi -1 1 ?f)",
      |subst| {
        not_free_in(Index(0), subst["f"])
      })
}

fn eta_expansion() -> Rewrite<Rise> {
    Rewrite::new("eta-expansion",
      "?f",
      "(lam (app ?f %0))")
}

fn sig_unused() -> Rewrite<Rise> {
    Rewrite::new_if("sig-unused",
      "(sig ?i ?body ?e)",
      "?body",
      |subst| {
        range_not_free_in(subst["i"], subst["body"])
      })
}

fn phi_unused() -> Rewrite<Rise> {
    Rewrite::new_if("phi-unused",
      "(phi ?i ?j ?body)",
      "?body",
      |subst| {
        range_not_free_in(subst["j"], subst["body"])
      })
}

/*
TODO: sig-var-const + phi-var-const

fn let_var_same() -> Rewrite<Rise> {
    let pat = "(let $1 ?e (var $1))";
    let outpat = "?e";
    Rewrite::new("let-var-same", pat, outpat)
}

fn let_var_diff() -> Rewrite<Rise> {
    let pat = "(let $1 ?e (var $2))";
    let outpat = "(var $2)";
    Rewrite::new("let-var-diff", pat, outpat)
}

TODO: sig-lam + phi-lam

fn let_lam_diff() -> Rewrite<Rise> {
    let pat = "(let $1 ?e (lam $2 ?body))";
    let outpat = "(lam $2 (let $1 ?e ?body))";
    Rewrite::new_if("let-lam-diff", pat, outpat, |subst| {
        subst["body"].slots().contains(&Slot::numeric(1))
    })
}

fn let_lam_diff_unopt() -> Rewrite<Rise> {
    let pat = "(let $1 ?e (lam $2 ?body))";
    let outpat = "(lam $2 (let $1 ?e ?body))";
    Rewrite::new("let-lam-diff-unopt", pat, outpat)
}
*/

fn sig_app() -> Rewrite<Rise> {
    Rewrite::new_if("sig-app",
      "(sig ?i (app ?a1 ?a2) ?b)",
      "(app (sig ?i ?a1 ?b) (sig ?i ?a2 ?b))",
      |subst| {
        range_not_free_in(subst["i"], subst["a1"]) ||
        range_not_free_in(subst["i"], subst["a2"])
      })
}

fn phi_app() -> Rewrite<Rise> {
  Rewrite::new_if("phi-app",
    "(phi ?i ?k (app ?a ?b))",
    "(app (phi ?i ?k ?a) (phi ?i ?k ?b))",
    |subst| {
      range_not_free_in(subst["k"], subst["a"]) ||
      range_not_free_in(subst["k"], subst["b"])
    })
}

/*
fn let_const() -> Rewrite<Rise> {
    // is the const-detection at the same time as the baseline? probably not relevant.
    let pat = Pattern::parse("(let $1 ?t ?c)").unwrap();

    let rt: RewriteT<Rise, (), ()> = RewriteT {
        searcher: Box::new(|_| ()),
        applier: Box::new(move |(), eg| {
            let span = tracing::trace_span!("let_const apply").entered();
            for subst in ematch_all(eg, &pat) {
                if eg.enodes_applied(&subst["c"]).iter().any(|n| matches!(n, Rise::Symbol(_) | Rise::Number(_))) {
                    let orig = pattern_subst(eg, &pat, &subst);
                    eg.union_justified(&orig, &subst["c"], Some("let-const".to_string()));
                }
            }
            span.exit();
        }),
    };
    rt.into()
}
*/
/////////////////////

fn map_fusion() -> Rewrite<Rise> {
    Rewrite::new("map-fusion",
      "(app (app map ?f) (app (app map ?g) ?arg))",
      "(app (app map (lam (app (phi 1 0 ?f) (app (phi 1 0 ?g) %0)))) ?arg)")
}

fn map_fission() -> Rewrite<Rise> {
    Rewrite::new_if("map-fission",
      "(app map (lam (app ?f ?gx)))",
      "(lam (app (app map ?f) (app (app map (lam (phi 1 1 ?gx))) %0)))",
      |subst| {
        not_free_in(Index(0), subst["f"])
      })
}
