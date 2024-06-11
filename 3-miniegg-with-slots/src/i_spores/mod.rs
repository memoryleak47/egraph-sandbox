use crate::*;

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum SporesENode {
    BigSum(Vec<Slot>, AppliedId),
    Add(AppliedId, AppliedId),
    Mul(AppliedId, AppliedId),
    Bind(Vec<Slot>, AppliedId),
    Unbind(Vec<Slot>, AppliedId),
    Matrix(Symbol, /*Shape*/),
}

impl Language for SporesENode {
    fn all_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        let mut out = Vec::new();
        match self {
            _ => todo!(),
        }
        out
    }

    fn public_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        let mut out = Vec::new();
        match self {
            _ => todo!(),
        }
        out
    }

    fn applied_id_occurences_mut(&mut self) -> Vec<&mut AppliedId> {
        match self {
            _ => todo!(),
        }
    }
}
