use crate::*;

pub fn rise_rules() -> Vec<Rewrite<Rise>> {
    let mut rewrites = Vec::new();

    rewrites.push(beta());
    rewrites.push(eta());
    // rewrites.push(eta_expansion());

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

fn beta() -> Rewrite<Rise> {
    let pat = "(app (lam $1 ?body) ?e)";
    let outpat = "?body[(var $1) := ?e]";

    Rewrite::new("beta", pat, outpat)
}

fn eta() -> Rewrite<Rise> {
    let pat = "(lam $1 (app ?f (var $1)))";
    let outpat = "?f";

    Rewrite::new_if("eta", pat, outpat, |subst| {
        !subst["f"].slots().contains(&Slot::numeric(1))
    })
}

fn eta_expansion() -> Rewrite<Rise> {
    let pat = "?f";
    let outpat = "(lam $1 (app ?f (var $1)))";

    Rewrite::new("eta-expansion", pat, outpat)
}

/////////////////////

fn map_fusion() -> Rewrite<Rise> {
    let mfu = "$0";
    let pat = "(app (app map ?f) (app (app map ?g) ?arg))";
    let outpat = &format!("(app (app map (lam {mfu} (app ?f (app ?g (var {mfu}))))) ?arg)");
    Rewrite::new("map-fusion", pat, outpat)
}

fn map_fission() -> Rewrite<Rise> {
    let x = 0;
    let mfi = 1;

    let pat = &format!(
        "(app map (lam ${x} (app ?f ?gx)))"
    );

    let outpat = &format!(
        "(lam ${mfi} (app (app map ?f) (app (app map (lam ${x} ?gx)) (var ${mfi}))))"
    );

    Rewrite::new_if("map-fission", pat, outpat, move |subst| {
        !subst["f"].slots().contains(&Slot::numeric(x))
    })
}

fn remove_transpose_pair() -> Rewrite<Rise> {
    let pat = "(app transpose (app transpose ?x))";
    let outpat = "?x";
    Rewrite::new("remove-transpose-pair", pat, outpat)
}

fn slide_before_map() -> Rewrite<Rise> {
    let pat = "(app (app (app slide ?sz) ?sp) (app (app map ?f) ?y))";
    let outpat = "(app (app map (app map ?f)) (app (app (app slide ?sz) ?sp) ?y))";
    Rewrite::new("slide-before-map", pat, outpat)
}

fn map_slide_before_transpose() -> Rewrite<Rise> {
    let pat = "(app transpose (app (app map (app (app slide ?sz) ?sp)) ?y))";
    let outpat = "(app (app map transpose) (app (app (app slide ?sz) ?sp) (app transpose ?y)))";
    Rewrite::new("map-slide-before-transpose", pat, outpat)
}

fn slide_before_map_map_f() -> Rewrite<Rise> {
    let pat = "(app (app map (app map ?f)) (app (app (app slide ?sz) ?sp) ?y))";
    let outpat = "(app (app (app slide ?sz) ?sp) (app (app map ?f) ?y))";
    Rewrite::new("slide-before-map-map-f", pat, outpat)
}

fn separate_dot_vh_simplified() -> Rewrite<Rise> {
    let x = "$0";
    let sdvh = "$1";

    let pat = &format!(
        "(app (app (app reduce add) 0) (app (app map (lam {x} (app (app mul (app fst (var {x}))) (app snd (var {x})))))
         (app (app zip (app join weights2d)) (app join ?nbh))))
        ");
    let outpat = &format!(
        "(app (app (app reduce add) 0) (app (app map (lam {x} (app (app mul (app fst (var {x}))) (app snd (var {x})))))
         (app (app zip weightsH) (app (app map (lam {sdvh} (app (app (app reduce add) 0) (app (app map (lam {x} (app (app mul (app fst (var {x}))) (app snd (var {x})))))
         (app (app zip weightsV) (var {sdvh})))))) (app transpose ?nbh)))))
        ");
    Rewrite::new("separate-dot-vh-simplified", pat, outpat)
}

fn separate_dot_hv_simplified() -> Rewrite<Rise> {
    let x = "$0";
    let sdhv = "$1";

    let pat = &format!(
        "(app (app (app reduce add) 0) (app (app map (lam {x} (app (app mul (app fst (var {x}))) (app snd (var {x})))))
         (app (app zip (app join weights2d)) (app join ?nbh))))
        ");
    let outpat = &format!(
        "(app (app (app reduce add) 0) (app (app map (lam {x} (app (app mul (app fst (var {x}))) (app snd (var {x})))))
         (app (app zip weightsV) (app (app map (lam {sdhv} (app (app (app reduce add) 0) (app (app map (lam {x} (app (app mul (app fst (var {x}))) (app snd (var {x})))))
         (app (app zip weightsH) (var {sdhv})))))) ?nbh))))
        ");

    Rewrite::new("separate-dot-hv-simplified", pat, outpat)
}
