use crate::*;

use std::marker::PhantomData;

pub trait CostFunction<L: Language> {
    type Cost: Ord + Clone;
    fn cost<C>(enode: &L, costs: C) -> Self::Cost where C: Fn(Id) -> Self::Cost;
}

pub struct AstSize<L: Language>(PhantomData<L>);

impl<L: Language> CostFunction<L> for AstSize<L> {
    type Cost = u64;

    fn cost<C>(enode: &L, costs: C) -> u64 where C: Fn(Id) -> u64 {
        let mut s: u64 = 1;
        for x in enode.applied_id_occurences() {
            s = s.saturating_add(costs(x.id));
        }
        s
    }
}
