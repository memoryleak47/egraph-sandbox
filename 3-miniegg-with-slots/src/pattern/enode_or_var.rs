use crate::*;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum ENodeOrVar<L: Language> {
    ENode(L),
    Var(String),
}

impl<L: Language> Language for ENodeOrVar<L> {
    fn discr(&self) -> u32 {
        match self {
            ENodeOrVar::ENode(x) => x.discr() + 1,
            ENodeOrVar::Var(_) => 0,
        }
    }

    fn all_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        match self {
            ENodeOrVar::ENode(x) => x.all_slot_occurences_mut(),
            ENodeOrVar::Var(_) => vec![],
        }
    }

    fn public_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        match self {
            ENodeOrVar::ENode(x) => x.public_slot_occurences_mut(),
            ENodeOrVar::Var(_) => vec![],
        }
    }

    fn applied_id_occurences_mut(&mut self) -> Vec<&mut AppliedId> {
        match self {
            ENodeOrVar::ENode(x) => x.applied_id_occurences_mut(),
            ENodeOrVar::Var(_) => vec![],
        }
    }


}
