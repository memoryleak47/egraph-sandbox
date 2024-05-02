use crate::*;

mod enode_or_var;
pub use enode_or_var::*;

mod ematch;
pub use ematch::*;

// The AppliedIds in `node` are ignored. They are replaced by the children RecExpr2.
// A non-fancy version of RecExpr that uses the slots as "names".
#[derive(Clone, PartialEq, Eq)]
pub struct RecExpr2<L: Language> {
    pub node: L,
    pub children: Vec<RecExpr2<L>>,
}

pub type Pattern<L> = RecExpr2<ENodeOrVar<L>>;
