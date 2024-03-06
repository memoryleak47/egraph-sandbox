use crate::*;

// TODO make complete.
#[derive(Clone, Debug)]
pub struct PermGroup {
    omega: Vec<Slot>, // the set of slots we are working on. Ordered lexicographically.
    index: usize, // where in omega we are.
    orbit: HashMap<Slot, SlotMap>, // orbit[k] maps omega[index] to k.
    sub: Option<Box<PermGroup>>,
}

impl PermGroup {
    pub fn identity(set: &HashSet<Slot>) -> Self {
        todo!()
    }
}
