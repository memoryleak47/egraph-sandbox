use crate::*;

// advanced functions:

pub fn add0() -> Pattern<ArithENode> { symb("add") }
pub fn add1(x: Pattern<ArithENode>) -> Pattern<ArithENode> { app(add0(), x) }
pub fn add2(x: Pattern<ArithENode>, y: Pattern<ArithENode>) -> Pattern<ArithENode> { app(add1(x), y) }

pub fn mul0() -> Pattern<ArithENode> { symb("mul") }
pub fn mul1(x: Pattern<ArithENode>) -> Pattern<ArithENode> { app(mul0(), x) }
pub fn mul2(x: Pattern<ArithENode>, y: Pattern<ArithENode>) -> Pattern<ArithENode> { app(mul1(x), y) }

// base functions:

pub fn pvar(s: &str) -> Pattern<ArithENode> {
    Pattern {
        node: ENodeOrPVar::PVar(s.to_string()),
        children: vec![],
    }
}

pub fn app(l: Pattern<ArithENode>, r: Pattern<ArithENode>) -> Pattern<ArithENode> {
    Pattern {
        node: ENodeOrPVar::ENode(ArithENode::App(empty_app_id(), empty_app_id())),
        children: vec![l, r],
    }
}

pub fn var(s: usize) -> Pattern<ArithENode> {
    var_slot(Slot::new(s))
}

pub fn var_slot(s: Slot) -> Pattern<ArithENode> {
    Pattern {
        node: ENodeOrPVar::ENode(ArithENode::Var(s)),
        children: vec![],
    }
}

pub fn lam(s: usize, b: Pattern<ArithENode>) -> Pattern<ArithENode> {
    lam_slot(Slot::new(s), b)
}

pub fn lam_slot(s: Slot, b: Pattern<ArithENode>) -> Pattern<ArithENode> {
    Pattern {
        node: ENodeOrPVar::ENode(ArithENode::Lam(s, empty_app_id())),
        children: vec![b],
    }
}

pub fn let_(s: usize, t: Pattern<ArithENode>, b: Pattern<ArithENode>) -> Pattern<ArithENode> {
    Pattern {
        node: ENodeOrPVar::ENode(ArithENode::Let(Slot::new(s), empty_app_id(), empty_app_id())),
        children: vec![t, b],
    }
}

pub fn num(i: u32) -> Pattern<ArithENode> {
    Pattern {
        node: ENodeOrPVar::ENode(ArithENode::Number(i)),
        children: vec![],
    }
}

pub fn symb(s: &str) -> Pattern<ArithENode> {
    Pattern {
        node: ENodeOrPVar::ENode(ArithENode::Symbol(Symbol::new(s))),
        children: vec![],
    }
}

// helper:

fn empty_app_id() -> AppliedId { AppliedId::new(Id(0), SlotMap::new()) }
