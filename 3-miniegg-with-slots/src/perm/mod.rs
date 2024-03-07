use crate::*;

mod stabchain;
mod cringe;

pub use cringe::*;

impl PermGroup {
    // Changes all Slot names contained in self using the `renamer`.
    pub fn rename_slots(&self, renamer: &Bijection) -> Self {
        assert_eq!(renamer.keys(), self.omega);

        let new_omega = renamer.values();

        let f = |x: &SlotMap| x.iter().map(|(x, y)| (renamer[x], renamer[y])).collect();
        let new_perms = self.generators().iter().map(f).collect();

        Self::new(&new_omega, new_perms)
    }
}
