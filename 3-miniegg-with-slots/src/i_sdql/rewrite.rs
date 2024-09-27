use crate::*;

pub fn sdql_rules() -> Vec<Rewrite<SdqlEnode>> {
    let pat = Pattern::parse("(sum s0 s1 ?R (sing ?e1 ?e2))").unwrap();
    let outpat = Pattern::parse("(sing ?e1 (sum s0 s1 ?R ?e2))").unwrap();

    let rewr = mk_rewrite_if(pat, outpat, |subst| {
        !subst["e1"].slots().contains(&Slot::new(0))
        && !subst["e1"].slots().contains(&Slot::new(1))
    });

    vec![rewr]

    //rw!("sum-fact-3";  "(sum ?R (sing ?e1 ?e2))"        => 
    //        { with_shifted_double_down(var("?e1"), var("?e1d"), 2, "(sing ?e1d (sum ?R ?e2))".parse::<Pattern<SDQL>>().unwrap()) }
    //        if and(neg(contains_ident(var("?e1"), Index(0))), neg(contains_ident(var("?e1"), Index(1))))),
}

#[test]
fn testy() {
    let R = "s0";
    let a = "s1";
    let i = "s2";
    let j = "s3";
    let input = &format!("(lambda {R} (lambda {a} (sum {i} {j} (var {R}) (sing (var {a}) (var {j})))))");
    let re: RecExpr<SdqlEnode> = RecExpr::parse(input).unwrap();
    let rewrites = sdql_rules();

    let mut eg = EGraph::new();

    let id = eg.add_expr(re.clone());

    do_rewrites(&mut eg, &rewrites);
    let term = extract::<_, AstSize>(id.clone(), &eg);
    eprintln!("{}", re.to_string());
    panic!("{}", term.to_string());
    
}
