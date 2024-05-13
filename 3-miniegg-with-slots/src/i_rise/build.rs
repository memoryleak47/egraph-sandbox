use crate::*;

// aux functions.
pub fn pvar_pat(s: &str) -> Pattern<RiseENode> {
    Pattern {
        node: ENodeOrPVar::PVar(s.to_string()),
        children: vec![],
    }
}

pub fn app_pat(l: Pattern<RiseENode>, r: Pattern<RiseENode>) -> Pattern<RiseENode> {
    Pattern {
        node: ENodeOrPVar::ENode(RiseENode::App(empty_app_id(), empty_app_id())),
        children: vec![l, r],
    }
}

pub fn var_pat(s: usize) -> Pattern<RiseENode> {
    Pattern {
        node: ENodeOrPVar::ENode(RiseENode::Var(Slot::new(s))),
        children: vec![],
    }
}

pub fn lam_pat(s: usize, b: Pattern<RiseENode>) -> Pattern<RiseENode> {
    Pattern {
        node: ENodeOrPVar::ENode(RiseENode::Lam(Slot::new(s), empty_app_id())),
        children: vec![b],
    }
}

pub fn let_pat(s: usize, t: Pattern<RiseENode>, b: Pattern<RiseENode>) -> Pattern<RiseENode> {
    Pattern {
        node: ENodeOrPVar::ENode(RiseENode::Let(Slot::new(s), empty_app_id(), empty_app_id())),
        children: vec![t, b],
    }
}

pub fn add_pat(l: Pattern<RiseENode>, r: Pattern<RiseENode>) -> Pattern<RiseENode> {
    Pattern {
        node: ENodeOrPVar::ENode(RiseENode::Add(empty_app_id(), empty_app_id())),
        children: vec![l, r],
    }
}

pub fn num_pat(i: u32) -> Pattern<RiseENode> {
    Pattern {
        node: ENodeOrPVar::ENode(RiseENode::Number(i)),
        children: vec![],
    }
}

fn empty_app_id() -> AppliedId { AppliedId::new(Id(0), SlotMap::new()) }

// recexpr2 functions:
pub fn app_re(l: RecExpr2<RiseENode>, r: RecExpr2<RiseENode>) -> RecExpr2<RiseENode> {
    RecExpr2 {
        node: RiseENode::App(empty_app_id(), empty_app_id()),
        children: vec![l, r],
    }
}

pub fn var_re(s: usize) -> RecExpr2<RiseENode> {
    RecExpr2 {
        node: RiseENode::Var(Slot::new(s)),
        children: vec![],
    }
}

pub fn lam_re(s: usize, b: RecExpr2<RiseENode>) -> RecExpr2<RiseENode> {
    RecExpr2 {
        node: RiseENode::Lam(Slot::new(s), empty_app_id()),
        children: vec![b],
    }
}

pub fn let_re(s: usize, t: RecExpr2<RiseENode>, b: RecExpr2<RiseENode>) -> RecExpr2<RiseENode> {
    RecExpr2 {
        node: RiseENode::Let(Slot::new(s), empty_app_id(), empty_app_id()),
        children: vec![t, b],
    }
}

pub fn add_re(l: RecExpr2<RiseENode>, r: RecExpr2<RiseENode>) -> RecExpr2<RiseENode> {
    RecExpr2 {
        node: RiseENode::Add(empty_app_id(), empty_app_id()),
        children: vec![l, r],
    }
}

pub fn num_re(i: u32) -> RecExpr2<RiseENode> {
    RecExpr2 {
        node: RiseENode::Number(i),
        children: vec![],
    }
}
