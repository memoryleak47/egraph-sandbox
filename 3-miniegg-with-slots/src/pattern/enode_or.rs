use crate::*;

pub type Pattern<L> = RecExpr<ENodeOrPVar<L>>;
pub type SemiRecExpr<L> = RecExpr<ENodeOrAppId<L>>;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
// PVar = pattern variable.
pub enum ENodeOrPVar<L: Language> {
    ENode(L),
    PVar(String),
}

impl<L: Language> Language for ENodeOrPVar<L> {
    fn all_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        match self {
            ENodeOrPVar::ENode(x) => x.all_slot_occurences_mut(),
            ENodeOrPVar::PVar(_) => vec![],
        }
    }

    fn public_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        match self {
            ENodeOrPVar::ENode(x) => x.public_slot_occurences_mut(),
            ENodeOrPVar::PVar(_) => vec![],
        }
    }

    fn applied_id_occurences_mut(&mut self) -> Vec<&mut AppliedId> {
        match self {
            ENodeOrPVar::ENode(x) => x.applied_id_occurences_mut(),
            ENodeOrPVar::PVar(_) => vec![],
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum ENodeOrAppId<L: Language> {
    ENode(L),
    AppliedId(AppliedId),
}

#[track_caller]
fn panic() -> ! {
    panic!("Pattern match on this! Don't use these methods")
}

impl<L: Language> Language for ENodeOrAppId<L> {
    fn all_slot_occurences_mut(&mut self) -> Vec<&mut Slot> { panic() }
    fn public_slot_occurences_mut(&mut self) -> Vec<&mut Slot> { panic() }
    fn applied_id_occurences_mut(&mut self) -> Vec<&mut AppliedId> { panic() }
}
