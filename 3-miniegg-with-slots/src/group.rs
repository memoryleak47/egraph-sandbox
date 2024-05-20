use crate::*;

// Trivial implementation of permutation Group.
#[derive(Clone, Debug)]
pub struct Group {
    // all perms are bijections : omega -> omega.
    pub omega: HashSet<Slot>,

    perms: HashSet<Perm>,
}

impl Group {
    pub fn identity(omega: &HashSet<Slot>) -> Self {
        Self {
            omega: omega.clone(),
            perms: singleton_set(SlotMap::identity(omega)),
        }
    }

    pub fn new(omega: &HashSet<Slot>, mut generators: HashSet<Perm>) -> Self {
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

        Group {
            omega: omega.clone(),
            perms: generators,
        }
    }

    pub fn union(&self, other: &Group) -> Group {
        assert_eq!(self.omega, other.omega);

        let perms = self.perms.union(&other.perms).cloned().collect();

        Self::new(&self.omega, perms)
    }

    pub fn intersection(&self, other: &Group) -> Group {
        assert_eq!(self.omega, other.omega);

        let perms = self.perms.intersection(&other.perms).cloned().collect();

        Self::new(&self.omega, perms)
    }

    // in the actual implementation, this would return a slightly smaller set.
    pub fn generators(&self) -> HashSet<Perm> {
        self.perms.clone()
    }

    pub fn omega(&self) -> HashSet<Slot> {
        self.omega.clone()
    }
}

pub fn singleton_set<T: Eq + std::hash::Hash>(t: T) -> HashSet<T> {
    [t].into_iter().collect()
}

#[test]
fn chaos_test() {
    let n = 5;
    let omega: HashSet<_> = (0..n).map(Slot).collect();

    let a = |x| (x+1)%n;
    let a: Perm = (0..n).map(|x| (Slot(x), Slot(a(x)))).collect();

    let b = |x| if x < 2 { 1 - x } else { x };
    let b: Perm = (0..n).map(|x| (Slot(x), Slot(b(x)))).collect();

    let perms = HashSet::from([a, b]);

    let grp = Group::new(&omega, perms);

    let fak = |x| (1..=x).product();
    assert_eq!(grp.generators().len(), fak(n));
}
