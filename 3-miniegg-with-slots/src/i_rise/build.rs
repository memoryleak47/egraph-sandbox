use crate::*;

// advanced functions:
pub fn map0() -> Pattern<RiseENode> { symb("map") }
pub fn map1(x: Pattern<RiseENode>) -> Pattern<RiseENode> { app(map0(), x) }
pub fn map2(x: Pattern<RiseENode>, y: Pattern<RiseENode>) -> Pattern<RiseENode> { app(map1(x), y) }

pub fn transpose0() -> Pattern<RiseENode> { symb("transpose") }
pub fn transpose1(x: Pattern<RiseENode>) -> Pattern<RiseENode> { app(transpose0(), x) }

pub fn slide0() -> Pattern<RiseENode> { symb("slide") }
pub fn slide1(x: Pattern<RiseENode>) -> Pattern<RiseENode> { app(slide0(), x) }
pub fn slide2(x: Pattern<RiseENode>, y: Pattern<RiseENode>) -> Pattern<RiseENode> { app(slide1(x), y) }
pub fn slide3(x: Pattern<RiseENode>, y: Pattern<RiseENode>, z: Pattern<RiseENode>) -> Pattern<RiseENode> { app(slide2(x, y), z) }


// base functions:

pub fn pvar(s: &str) -> Pattern<RiseENode> {
    Pattern {
        node: ENodeOrPVar::PVar(s.to_string()),
        children: vec![],
    }
}

pub fn app(l: Pattern<RiseENode>, r: Pattern<RiseENode>) -> Pattern<RiseENode> {
    Pattern {
        node: ENodeOrPVar::ENode(RiseENode::App(empty_app_id(), empty_app_id())),
        children: vec![l, r],
    }
}

pub fn var(s: usize) -> Pattern<RiseENode> {
    Pattern {
        node: ENodeOrPVar::ENode(RiseENode::Var(Slot::new(s))),
        children: vec![],
    }
}

pub fn lam(s: usize, b: Pattern<RiseENode>) -> Pattern<RiseENode> {
    Pattern {
        node: ENodeOrPVar::ENode(RiseENode::Lam(Slot::new(s), empty_app_id())),
        children: vec![b],
    }
}

pub fn let_(s: usize, t: Pattern<RiseENode>, b: Pattern<RiseENode>) -> Pattern<RiseENode> {
    Pattern {
        node: ENodeOrPVar::ENode(RiseENode::Let(Slot::new(s), empty_app_id(), empty_app_id())),
        children: vec![t, b],
    }
}

pub fn add(l: Pattern<RiseENode>, r: Pattern<RiseENode>) -> Pattern<RiseENode> {
    Pattern {
        node: ENodeOrPVar::ENode(RiseENode::Add(empty_app_id(), empty_app_id())),
        children: vec![l, r],
    }
}

pub fn num(i: u32) -> Pattern<RiseENode> {
    Pattern {
        node: ENodeOrPVar::ENode(RiseENode::Number(i)),
        children: vec![],
    }
}

pub fn symb(s: &str) -> Pattern<RiseENode> {
    Pattern {
        node: ENodeOrPVar::ENode(RiseENode::Symbol(Symbol::new(s))),
        children: vec![],
    }
}

// helper:

fn empty_app_id() -> AppliedId { AppliedId::new(Id(0), SlotMap::new()) }
