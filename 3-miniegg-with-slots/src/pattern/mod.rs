use crate::*;

// The AppliedIds in L are ignored. They are replaced by the children RecExpr2.
pub struct RecExpr2<L: Language> {
    node: L,
    children: Vec<RecExpr2<L>>,
}



