use crate::*;
use std::fmt::*;

impl Debug for Slot {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.to_string())
    }
}

impl Debug for Id {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "id{}", self.0)
    }
}

impl Debug for SlotMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "[")?;
        let n = self.len();
        for (i, (x, y)) in self.iter().enumerate() {
            write!(f, "{x:?} -> {y:?}")?;
            if i < n-1 {
                write!(f, ", ")?;
            }
        }
        write!(f, "]")
    }
}

impl Debug for AppliedId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:?}{:?}", self.id, self.m)
    }
}

impl<L: Language> Debug for Explanation<L> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "- {}\n", &self.term)?;
        if let Some(step) = &self.step {
            write!(f, "{:?}", step)?;
        }
        Ok(())
    }
}

impl<L: Language> Debug for ExplanationStep<L> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "-- by {:?} at {:?}\n", &self.justification, &self.index_list)?;
        write!(f, "{:?}", &self.exp)
    }
}

