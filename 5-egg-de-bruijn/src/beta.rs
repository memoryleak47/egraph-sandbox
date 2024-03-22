use crate::*;

pub fn beta_reduction() -> Rewrite<ENode, ()> {
    rewrite!("beta-reduction"; "(app (lam ?b) ?t)" => { BetaReduction })
}

struct BetaReduction;

impl Applier<ENode, ()> for BetaReduction {
    fn apply_one(&self, eg: &mut EG, id: Id, subst: &Subst, _pat: Option<&PatternAst<ENode>>, _rule_name: Symbol) -> Vec<Id> {
        let b: Var = "?b".parse().unwrap();
        let t: Var = "?t".parse().unwrap();

        let b: Id = subst[b];
        let t: Id = subst[t];

        let new = substitution(b, t, eg);
        eg.union(new, id);

        Vec::new() // is this fine?
    }
}

fn substitution(b: Id, t: Id, eg: &mut EG) -> Id {
    todo!()
}
