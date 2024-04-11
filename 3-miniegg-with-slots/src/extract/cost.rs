use crate::*;

use std::marker::PhantomData;

pub trait CostFn<L: Language> {
    fn cost<C>(enode: &L, costs: C) -> u64 where C: Fn(Id) -> u64;
}

pub struct AstSize<L: Language>(PhantomData<L>);

impl<L: Language> CostFn<L> for AstSize<L> {
    fn cost<C>(enode: &L, costs: C) -> u64 where C: Fn(Id) -> u64 {
        let mut s: u64 = 1;
        for x in enode.applied_id_occurences() {
            s = s.saturating_add(costs(x.id));
        }
        s
    }
}
