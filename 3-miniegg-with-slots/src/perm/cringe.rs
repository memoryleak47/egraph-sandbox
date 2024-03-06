use crate::*;

// Cringe implementations of PermGroup.
#[derive(Clone, Debug)]
pub struct PermGroup {
    // all perms are bijections : omega -> omega.
    omega: HashSet<Slot>,

    perms: HashSet<SlotMap>,
}

impl PermGroup {
    pub fn identity(omega: &HashSet<Slot>) -> Self {
        Self {
            omega: omega.clone(),
            perms: HashSet::from([SlotMap::identity(omega)])
        }
    }

    pub fn new(omega: &HashSet<Slot>, mut generators: HashSet<SlotMap>) -> Self {
        for x in &generators {
            assert!(x.is_bijection());
            assert_eq!(&x.keys(), omega);
            assert_eq!(&x.values(), omega);
        }

        generators.insert(SlotMap::identity(omega));
        
        loop {
            let copy = generators.clone();

            for x in &copy {
                let new = x.inverse();
                generators.insert(new);
            }

            for x in &copy {
                for y in &copy {
                    let new = x.compose(y);
                    generators.insert(new);
                }
            }

            if generators.len() == copy.len() { break; }
        }

        PermGroup {
            omega: omega.clone(),
            perms: generators,
        }
    }

    pub fn union(&self, other: &PermGroup) -> PermGroup {
        assert_eq!(self.omega, other.omega);

        let perms = self.perms.union(&other.perms).cloned().collect();

        Self::new(&self.omega, perms)
    }

    pub fn intersection(&self, other: &PermGroup) -> PermGroup {
        assert_eq!(self.omega, other.omega);

        let perms = self.perms.intersection(&other.perms).cloned().collect();

        Self::new(&self.omega, perms)
    }
}
