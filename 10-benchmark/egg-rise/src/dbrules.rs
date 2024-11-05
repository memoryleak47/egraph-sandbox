use egg::*;
use crate::dbrise::*;
use std::collections::HashMap;

fn var(s: &str) -> Var {
    s.parse().unwrap()
}

fn var_to_num(v: Var) -> impl Fn(&mut DBRiseEGraph, Id, &Subst) -> i64 {
    move |egraph, _, subst| {
        for x in egraph[subst[v]].nodes.iter() {
            let DBRise::Number(i) = x else { continue };
            return *i as i64;
        }
        panic!();
    }
}

fn contains_ident(v1: Var, index: Index) -> impl Fn(&mut DBRiseEGraph, Id, &Subst) -> bool {
    move |egraph, _, subst| egraph[subst[v1]].data.free.contains(&index)
}

// checks whether v2 or something bigger comes up in b, or in other words whether removing v2 would index shift something in v2.
fn in_range(b: Var, v2: Var) -> impl Fn(&mut DBRiseEGraph, Id, &Subst) -> bool {
    move |egraph, a, subst| {
        let idx = var_to_num(v2)(egraph, a, subst);
        let b_max = egraph[subst[b]].data.free.iter().max();
        if let Some(b_x) = b_max {
            b_x.0 as i64 >= idx
        } else { false }
    }
}

fn neg(f: impl Fn(&mut DBRiseEGraph, Id, &Subst) -> bool) -> impl Fn(&mut DBRiseEGraph, Id, &Subst) -> bool {
    move |egraph, id, subst| !f(egraph, id, subst)
}

fn and(f1: impl Fn(&mut DBRiseEGraph, Id, &Subst) -> bool, f2: impl Fn(&mut DBRiseEGraph, Id, &Subst) -> bool) -> impl Fn(&mut DBRiseEGraph, Id, &Subst) -> bool {
    move |egraph, id, subst| f1(egraph, id, subst) && f2(egraph, id, subst)
}

fn or(f1: impl Fn(&mut DBRiseEGraph, Id, &Subst) -> bool, f2: impl Fn(&mut DBRiseEGraph, Id, &Subst) -> bool) -> impl Fn(&mut DBRiseEGraph, Id, &Subst) -> bool {
    move |egraph, id, subst| f1(egraph, id, subst) || f2(egraph, id, subst)
}

pub fn dbrules(names: &[&str]) -> Vec<Rewrite<DBRise, DBRiseAnalysis>> {
    let all_rules = vec![
        // reductions
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
            "(app (app (app reduce add) 0) (app (app map (lam (app (app mul (app fst %0)) (app snd %0))))
             (app (app zip (app join weights2d)) (app join ?nbh))))" =>
            "(app (app (app reduce add) 0) (app (app map (lam (app (app mul (app fst %0)) (app snd %0))))
             (app (app zip weightsV) (app (app map (lam (app (app (app reduce add) 0) (app (app map (lam (app (app mul (app fst %0)) (app snd %0))))
             (app (app zip weightsH) %0))))) ?nbh))))"),
        rewrite!("separate-dot-vh-simplified";
            "(app (app (app reduce add) 0) (app (app map (lam (app (app mul (app fst %0)) (app snd %0))))
             (app (app zip (app join weights2d)) (app join ?nbh))))" =>
            "(app (app (app reduce add) 0) (app (app map (lam (app (app mul (app fst %0)) (app snd %0))))
             (app (app zip weightsH) (app (app map (lam (app (app (app reduce add) 0) (app (app map (lam (app (app mul (app fst %0)) (app snd %0))))
             (app (app zip weightsV) %0))))) (app transpose ?nbh)))))"),


        // algorithmic
        rewrite!("map-fusion";
            "(app (app map ?f) (app (app map ?g) ?arg))" =>
            "(app (app map (lam (app (phi 1 0 ?f) (app (phi 1 0 ?g) %0)))) ?arg)"),
        rewrite!("map-fission";
            "(app map (lam (app ?f ?gx)))" =>
            "(lam (app (app map ?f) (app (app map (lam (phi 1 1 ?gx))) %0)))"
            // TODO: if conditions should be recursive filters?
            if neg(contains_ident(var("?f"), Index(0)))),

        // reductions
        rewrite!("beta"; "(app (lam ?body) ?e)" => "(sig 0 ?body ?e)"),
        rewrite!("eta"; "(lam (app ?f %0))" => "(phi -1 1 ?f)"
            // TODO: if conditions should be recursive filters?
            if neg(contains_ident(var("?f"), Index(0)))),

        rewrite!("sig-unused"; "(sig ?i ?body ?e)" => "?body" if neg(in_range(var("?body"), var("?i")))),
        rewrite!("phi-unused"; "(phi ?i ?j ?body)" => "?body" if neg(in_range(var("?body"), var("?j")))),

        // explicit substitution / shifting
        rewrite!("sig-lam"; "(sig ?i (lam ?a) ?b)" =>
            { NumberShiftApplier { var: var("?i"), shift: 1, new_var: var("?ip1"),
              applier: "(lam (sig ?ip1 ?a ?b))".parse::<Pattern<DBRise>>().unwrap() } } if in_range(var("?a"), var("?i"))),
        rewrite!("sig-app"; "(sig ?i (app ?a1 ?a2) ?b)" => "(app (sig ?i ?a1 ?b) (sig ?i ?a2 ?b))" if or(in_range(var("?a1"), var("?i")), in_range(var("?a2"), var("?i")))),
        /* DEPRECATED: const < unused
        rewrite!("sig-var-const"; "(sig ?i ?n ?b)" =>
            { SigVarConstApplier { i: var("?i"), n: var("?n"), b: var("?b") }}), */
        rewrite!("sig-var"; "(sig ?i ?n ?b)" =>
            { SigVarApplier { i: var("?i"), n: var("?n"), b: var("?b") }}),
        rewrite!("phi-lam"; "(phi ?i ?k (lam ?a))" =>
            { NumberShiftApplier { var: var("?k"), shift: 1, new_var: var("?kp1"),
              applier: "(lam (phi ?i ?kp1 ?a))".parse::<Pattern<DBRise>>().unwrap() }} if in_range(var("?a"), var("?k"))),
        rewrite!("phi-app"; "(phi ?i ?k (app ?a ?b))" => "(app (phi ?i ?k ?a) (phi ?i ?k ?b))" if or(in_range(var("?a"), var("?k")), in_range(var("?b"), var("?k")))),
        /* DEPRECATED: const < unused
        rewrite!("phi-var-const"; "(phi ?i ?k ?n)" =>
            { PhiVarConstApplier { i: var("?i"), k: var("?k"), n: var("?n") }}),
             */
        rewrite!("phi-var"; "(phi ?i ?k ?n)" =>
            { PhiVarApplier { i: var("?i"), k: var("?k"), n: var("?n") }}),

        rewrite!("eta-expansion"; "?f" => "(lam (app (phi 1 0 ?f) %0))"),
    ];
    let mut map: HashMap<Symbol, _> = all_rules.into_iter().map(|r| (r.name.to_owned(), r)).collect();
    names.into_iter().map(|&n| map.remove(&Symbol::new(n)).expect("rule not found")).collect()
}

struct NumberShiftApplier<A> {
    var: Var,
    shift: i32,
    new_var: Var,
    applier: A,
}

impl<A> Applier<DBRise, DBRiseAnalysis> for NumberShiftApplier<A> where A: Applier<DBRise, DBRiseAnalysis> {
    fn apply_one(&self, egraph: &mut DBRiseEGraph, eclass: Id, subst: &Subst,
                 searcher_ast: Option<&PatternAst<DBRise>>, rule_name: Symbol) -> Vec<Id> {
        // TODO: use i32_from_eclass?
        let extract = &egraph[subst[self.var]].data.beta_extract;
        let shifted = match extract.as_ref() {
            [DBRise::Number(i)] => DBRise::Number(i + self.shift),
            _ => panic!()
        };
        let mut subst = subst.clone();
        subst.insert(self.new_var, egraph.add(shifted));
        self.applier.apply_one(egraph, eclass, &subst, searcher_ast, rule_name)
    }
}

struct SigVarApplier { // DEPRECATED: SigVarConstApplier
    i: Var,
    n: Var,
    b: Var,
}

impl Applier<DBRise, DBRiseAnalysis> for SigVarApplier {
    fn apply_one(&self, egraph: &mut DBRiseEGraph, eclass: Id, subst: &Subst,
                 _searcher_ast: Option<&PatternAst<DBRise>>, _rule_name: Symbol) -> Vec<Id> {
        match egraph[subst[self.n]].data.beta_extract.as_ref() {
            /* DEPRECATED: Const case
            [DBRise::Number(_)] | [DBRise::Symbol(_)] => {
                let id = subst[self.n];
                if egraph.union(eclass, id) {
                    vec![eclass]
                } else {
                    vec![]
                }
            } */
            &[DBRise::Var(Index(var))] => {
                let i_num = i32_from_eclass(egraph, subst[self.i]);
                let n = var as i32;
                let node = if n > i_num {
                    DBRise::Var(Index(var - 1))
                } else if n == i_num {
                    DBRise::Phi([subst[self.i], egraph.add(DBRise::Number(0)), subst[self.b]])
                } else { // n < i_num
                    DBRise::Var(Index(var))
                };
                let id = egraph.add(node);
                if egraph.union(eclass, id) {
                    vec![eclass]
                } else {
                    vec![]
                }
            }
            _ => vec![] // do nothing
        }
    }
}

struct PhiVarApplier { // DEPRECATED: PhiVarConstApplier
    i: Var,
    k: Var,
    n: Var,
}

impl Applier<DBRise, DBRiseAnalysis> for PhiVarApplier {
    fn apply_one(&self, egraph: &mut DBRiseEGraph, eclass: Id, subst: &Subst,
                 _searcher_ast: Option<&PatternAst<DBRise>>, _rule_name: Symbol) -> Vec<Id> {
        match egraph[subst[self.n]].data.beta_extract.as_ref() {
            /* DEPRECATED: Const case
            [DBRise::Number(_)] | [DBRise::Symbol(_)] => {
                let id = subst[self.n];
                if egraph.union(eclass, id) {
                    vec![eclass]
                } else {
                    vec![]
                }
            } */
            &[DBRise::Var(Index(var))] => {
                let i_num = i32_from_eclass(egraph, subst[self.i]);
                let k_num = i32_from_eclass(egraph, subst[self.k]);
                let n = var as i32;
                let shifted = DBRise::Var(Index(if n >= k_num { (n + i_num) as u32 } else { var }));
                let id = egraph.add(shifted);
                if egraph.union(eclass, id) {
                    vec![eclass]
                } else {
                    vec![]
                }
            }
            _ => vec![] // do nothing
        }
    }
}
