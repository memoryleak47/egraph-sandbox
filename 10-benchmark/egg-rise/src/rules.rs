use egg::*;
use crate::rise::*;
use std::collections::HashMap;

fn var(s: &str) -> Var {
    s.parse().unwrap()
}

fn is_not_same_var(v1: Var, v2: Var) -> impl Fn(&mut RiseEGraph, Id, &Subst) -> bool {
    move |egraph, _, subst| egraph.find(subst[v1]) != egraph.find(subst[v2])
}

fn contains_ident(v1: Var, v2: Var) -> impl Fn(&mut RiseEGraph, Id, &Subst) -> bool {
    move |egraph, _, subst| egraph[subst[v1]].data.free.contains(&subst[v2])
}

fn neg(f: impl Fn(&mut RiseEGraph, Id, &Subst) -> bool) -> impl Fn(&mut RiseEGraph, Id, &Subst) -> bool {
    move |egraph, id, subst| !f(egraph, id, subst)
}

fn and(f1: impl Fn(&mut RiseEGraph, Id, &Subst) -> bool, f2: impl Fn(&mut RiseEGraph, Id, &Subst) -> bool) -> impl Fn(&mut RiseEGraph, Id, &Subst) -> bool {
    move |egraph, id, subst| f1(egraph, id, subst) && f2(egraph, id, subst)
}

fn or(f1: impl Fn(&mut RiseEGraph, Id, &Subst) -> bool, f2: impl Fn(&mut RiseEGraph, Id, &Subst) -> bool) -> impl Fn(&mut RiseEGraph, Id, &Subst) -> bool {
    move |egraph, id, subst| f1(egraph, id, subst) || f2(egraph, id, subst)
}

pub fn rules(names: &[&str]) -> Vec<Rewrite<Rise, RiseAnalysis>> {
    let all_rules = vec![
        // algorithmic
        rewrite!("map-fusion";
            "(app (app map ?f) (app (app map ?g) ?arg))" =>
            { with_fresh_var("?mfu", "(app (app map (lam ?mfu (app ?f (app ?g (var ?mfu))))) ?arg)") }),
        rewrite!("map-fission";
            "(app map (lam ?x (app ?f ?gx)))" =>
            { with_fresh_var("?mfi", "(lam ?mfi (app (app map ?f) (app (app map (lam ?x ?gx)) (var ?mfi))))") }
            // TODO: if conditions should be recursive filters?
            if neg(contains_ident(var("?f"), var("?x")))),

        // reductions
        rewrite!("eta"; "(lam ?v (app ?f (var ?v)))" => "?f"
            // TODO: if conditions should be recursive filters?
            if neg(contains_ident(var("?f"), var("?v")))),
        rewrite!("remove-transpose-pair";
            "(app transpose (app transpose ?x))" => "?x"),

        // movement
        rewrite!("map-slide-before-transpose";
            "(app transpose (app (app map (app (app slide ?sz) ?sp)) ?y))" =>
            "(app (app map transpose) (app (app (app slide ?sz) ?sp) (app transpose ?y)))"),
        rewrite!("slide-before-map-map-f";
            "(app (app map (app map ?f)) (app (app (app slide ?sz) ?sp) ?y))" =>
            "(app (app (app slide ?sz) ?sp) (app (app map ?f) ?y))"),
        rewrite!("slide-before-map";
            "(app (app (app slide ?sz) ?sp) (app (app map ?f) ?y))" =>
            "(app (app map (app map ?f)) (app (app (app slide ?sz) ?sp) ?y))"),

        // domain-specific
        rewrite!("separate-dot-hv-simplified";
            "(app (app (app reduce add) 0) (app (app map (lam ?x (app (app mul (app fst (var ?x))) (app snd (var ?x)))))
             (app (app zip (app join weights2d)) (app join ?nbh))))" =>
            { with_fresh_var("?sdhv", "(app (app (app reduce add) 0) (app (app map (lam ?x (app (app mul (app fst (var ?x))) (app snd (var ?x)))))
                (app (app zip weightsV) (app (app map (lam ?sdhv (app (app (app reduce add) 0) (app (app map (lam ?x (app (app mul (app fst (var ?x))) (app snd (var ?x)))))
                (app (app zip weightsH) (var ?sdhv)))))) ?nbh))))") }),
        rewrite!("separate-dot-vh-simplified";
            "(app (app (app reduce add) 0) (app (app map (lam ?x (app (app mul (app fst (var ?x))) (app snd (var ?x)))))
             (app (app zip (app join weights2d)) (app join ?nbh))))" =>
            { with_fresh_var("?sdvh", "(app (app (app reduce add) 0) (app (app map (lam ?x (app (app mul (app fst (var ?x))) (app snd (var ?x)))))
                (app (app zip weightsH) (app (app map (lam ?sdvh (app (app (app reduce add) 0) (app (app map (lam ?x (app (app mul (app fst (var ?x))) (app snd (var ?x)))))
                (app (app zip weightsV) (var ?sdvh)))))) (app transpose ?nbh)))))") }),


        // GENERAL:
        rewrite!("beta"; "(app (lam ?v ?body) ?e)" => "(let ?v ?e ?body)"),

        // OPTIMIZED:
        rewrite!("opt:let-unused"; "(let ?v ?t ?body)" => "?body" if neg(contains_ident(var("?body"), var("?v")))),
        rewrite!("opt:let-app"; "(let ?v ?e (app ?a ?b))" => "(app (let ?v ?e ?a) (let ?v ?e ?b))" if or(contains_ident(var("?a"), var("?v")), contains_ident(var("?b"), var("?v")))),
        rewrite!("opt:let-var-same"; "(let ?v1 ?e (var ?v1))" => "?e"),
        rewrite!("opt:let-lam-same"; "(let ?v1 ?e (lam ?v1 ?body))" => "(lam ?v1 ?body)"),
        rewrite!("opt:let-lam-diff"; "(let ?v1 ?e (lam ?v2 ?body))" =>
           { CaptureAvoid {
               fresh: var("?fresh"), v2: var("?v2"), e: var("?e"),
               if_not_free: "(lam ?v2 (let ?v1 ?e ?body))".parse().unwrap(),
               if_free: "(lam ?fresh (let ?v1 ?e (let ?v2 (var ?fresh) ?body)))".parse().unwrap(),
           }}
           if and(is_not_same_var(var("?v1"), var("?v2")), contains_ident(var("?body"), var("?v1")))),

        rewrite!("eta-expansion"; "?f" => { with_fresh_var("?eexp", "(lam ?eexp (app ?f (var ?eexp)))") }),
    ];
    let mut map: HashMap<Symbol, _> = all_rules.into_iter().map(|r| (r.name.to_owned(), r)).collect();
    names.into_iter().map(|&n| map.remove(&Symbol::new(n)).expect("rule not found")).collect()
}

fn with_fresh_var(name: &str, pattern: &str) -> MakeFresh {
    MakeFresh {
        prefix: name[1..].into(),
        fresh: var(name),
        pattern: pattern.parse().unwrap()
    }
}

struct MakeFresh {
    prefix: String,
    fresh: Var,
    pattern: Pattern<Rise>,
}

impl Applier<Rise, RiseAnalysis> for MakeFresh {
    fn apply_one(&self, egraph: &mut RiseEGraph, eclass: Id, subst: &Subst,
                 searcher_ast: Option<&PatternAst<Rise>>, rule_name: Symbol) -> Vec<Id> {
        let sym = Rise::Symbol(format!("{}{}", self.prefix, eclass).into());
        let mut subst = subst.clone();
        subst.insert(self.fresh, egraph.add(sym));
        self.pattern.apply_one(egraph, eclass, &subst, searcher_ast, rule_name)
    }
}

struct CaptureAvoid {
    fresh: Var,
    v2: Var,
    e: Var,
    if_not_free: Pattern<Rise>,
    if_free: Pattern<Rise>,
}

impl Applier<Rise, RiseAnalysis> for CaptureAvoid {
    fn apply_one(
        &self,
        egraph: &mut RiseEGraph,
        eclass: Id,
        subst: &Subst,
        searcher_ast: Option<&PatternAst<Rise>>,
        rule_name: Symbol
    ) -> Vec<Id> {
        let e = subst[self.e];
        let v2 = subst[self.v2];
        let v2_free_in_e = egraph[e].data.free.contains(&v2);
        if v2_free_in_e {
            let mut subst = subst.clone();
            let sym = Rise::Symbol(format!("_{}", eclass).into());
            subst.insert(self.fresh, egraph.add(sym));
            self.if_free
                .apply_one(egraph, eclass, &subst, searcher_ast, rule_name)
        } else {
            self.if_not_free
                .apply_one(egraph, eclass, &subst, searcher_ast, rule_name)
        }
    }
}
