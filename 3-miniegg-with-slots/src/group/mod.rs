use crate::*;

use std::ops::Index;
use std::hash::Hash;

#[cfg(test)]
mod tst;

pub trait Permutation: Index<Slot, Output=Slot> + Clone + Eq + Hash {
    fn iter(&self) -> impl Iterator<Item=(Slot, Slot)>;
    fn identity(omega: &HashSet<Slot>) -> Self;
    fn compose(&self, other: &Self) -> Self;
    fn inverse(&self) -> Self;
}

impl Permutation for Perm {
    fn iter(&self) -> impl Iterator<Item=(Slot, Slot)> { Self::iter(self) }
    fn identity(omega: &HashSet<Slot>) -> Self { Self::identity(omega) }
    fn compose(&self, other: &Self) -> Self { Self::compose(self, other) }
    fn inverse(&self) -> Self { Self::inverse(self) }
}

// In order to be compatible with the literature:
// https://en.wikipedia.org/wiki/Schreier%27s_lemma
// I define "x y" = x.compose(y)

// Trivial implementation of permutation Group.
#[derive(Clone, Debug)]
pub struct Group<P: Permutation> {
    // all perms are bijections : omega -> omega.
    omega: HashSet<Slot>,
    next: Option<Box<Next<P>>>,
}

#[derive(Clone, Debug)]
struct Next<P: Permutation> {
    // the Slot we are stabilizing
    stab: Slot,

    // the orbit tree.
    // ot[x] is a perm that maps stab to x.
    ot: HashMap<Slot, P>,

    g: Group<P>,
}

impl<P: Permutation> Group<P> {
    pub fn new(omega: &HashSet<Slot>, generators: HashSet<P>) -> Self {
        let omega = omega.clone();
        let next = find_lowest_nonstab(&generators)
                    .map(|s| Box::new(Next::new(s, &omega, generators)));
        Group { omega, next }
    }

    pub fn identity(omega: &HashSet<Slot>) -> Self {
        Self::new(omega, HashSet::default())
    }

    pub fn orbit(&self, s: Slot) -> HashSet<Slot> {
        build_ot(s, &self.omega, &self.generators())
            .keys()
            .cloned()
            .collect()
    }

    fn generators_impl(&self) -> HashSet<P> {
        match &self.next {
            None => HashSet::default(),
            Some(n) => &n.ot.values().cloned().collect::<HashSet<_>>() | &n.g.generators_impl(),
        }
    }

    pub fn generators(&self) -> HashSet<P> {
        let mut out = self.generators_impl();
        out.remove(&P::identity(&self.omega));
        out
    }

    // Should be very rarely called.
    pub fn all_perms(&self) -> HashSet<P> {
        match &self.next {
            None => [P::identity(&self.omega)].into_iter().collect(),
            Some(n) => {
                let mut out = HashSet::default();

                let left = n.ot.values().cloned().collect::<HashSet<_>>();
                let right = n.g.all_perms();

                for l in &left {
                    for r in &right {
                        out.insert(r.compose(l));
                    }
                }

                if CHECKS {
                    assert_eq!(out.len(), self.count());
                }

                out
            }
        }
    }

    pub fn contains(&self, p: &P) -> bool {
        match &self.next {
            None => p.iter().all(|(x, y)| x == y),
            Some(n) => {
                let Some(part) = &n.ot.get(&p[n.stab]) else { return false };
                n.g.contains(&p.compose(&part.inverse()))
            },
        }
    }

    pub fn add(&mut self, p: P) {
        self.add_set([p].into_iter().collect());
    }

    pub fn add_set(&mut self, mut perms: HashSet<P>) {
        // There might be ways to make this faster, by iterating through the stab chain and determining at which layer this perm actually has an effect.
        // But it's polytime, so fast enough I guess.

        perms.retain(|x| !self.contains(x));

        if !perms.is_empty() {
            *self = Group::new(&self.omega, &self.generators() | &perms);
        }
    }

    pub fn count(&self) -> usize {
        match &self.next {
            None => 1,
            Some(n) => n.ot.len() * n.g.count(),
        }
    }
}

impl<P: Permutation> Next<P> {
    fn new(stab: Slot, omega: &HashSet<Slot>, generators: HashSet<P>) -> Self {
        let ot = build_ot(stab, omega, &generators);
        let generators = schreiers_lemma(stab, &ot, generators);
        let g = Group::new(omega, generators);
        Next { stab, ot, g }
    }
}

fn build_ot<P: Permutation>(stab: Slot, omega: &HashSet<Slot>, generators: &HashSet<P>) -> HashMap<Slot, P> {
    let mut ot = HashMap::default();
    ot.insert(stab, P::identity(omega));

    loop {
        let len = ot.len();

        for g in generators {
            for (_, v) in ot.clone() {
                let new = v.compose(g);
                let target = new[stab];
                if !ot.contains_key(&target) {
                    ot.insert(target, new);
                }
            }
        }

        let len2 = ot.len();
        if len == len2 { break; }
    }

    ot
}

// extends the set of generators using Schreiers Lemma.
fn schreiers_lemma<P: Permutation>(stab: Slot, ot: &HashMap<Slot, P>, generators: HashSet<P>) -> HashSet<P> {
    let mut out = HashSet::default();
    for (_, r) in ot {
        for s in &generators {
            let rs = r.compose(s);
            let rs2_inv = ot[&rs[stab]].inverse();
            out.insert(rs.compose(&rs2_inv));
        }
    }
    out
}

// finds the lowest Slot that's not stabilized in at least one of the generators.
fn find_lowest_nonstab<P: Permutation>(generators: &HashSet<P>) -> Option<Slot> {
    let mut min = None;
    for gen in generators {
        for (x, y) in gen.iter() {
            if x != y {
                min = min.iter()
                         .copied()
                         .chain(std::iter::once(x))
                         .min();
            }
        }
    }
    min
}

pub fn singleton_set<T: Eq + Hash>(t: T) -> HashSet<T> {
    [t].into_iter().collect()
}
