
use crate::*;

use std::cmp::Ordering;

// only returns valid lambda calculus terms, nothing else.
pub struct RestrictedAstSize;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MyCost {
    Finite(u32),
    Infinite,
}

fn add(a: MyCost, b: MyCost) -> MyCost {
    use MyCost::*;
    match (a, b) {
        (Infinite, _) => Infinite,
        (_, Infinite) => Infinite,
        (Finite(x), Finite(y)) => Finite(x + y),
    }
}

fn add1(a: MyCost, b: MyCost) -> MyCost {
    add(add(a, b), MyCost::Finite(1))
}

impl PartialOrd for MyCost {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use MyCost::*;
        match (self, other) {
            (Finite(x), Finite(y)) => x.partial_cmp(y),
            (Infinite, Infinite) => Some(Ordering::Equal),
            (Infinite, Finite(_)) => Some(Ordering::Greater),
            (Finite(_), Infinite) => Some(Ordering::Less),
        }
    }
}

impl CostFunction<Lambda> for RestrictedAstSize {
    type Cost = MyCost;

    fn cost<C>(&mut self, t: &Lambda, mut costs: C) -> MyCost where C: FnMut(Id) -> MyCost {
        match t {
            Lambda::Lambda([_, b]) => add(costs(*b), MyCost::Finite(1)),
            Lambda::Var(_) => MyCost::Finite(1), // TODO is this a monotonicity problem?
            Lambda::App([l, r]) => add1(costs(*l), costs(*r)),
            _ => MyCost::Infinite,
        }
    }
}


