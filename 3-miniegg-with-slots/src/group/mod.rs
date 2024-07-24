use crate::*;

#[cfg(test)]
mod tst;

// In order to be compatible with the literature:
// https://en.wikipedia.org/wiki/Schreier%27s_lemma
// I define "x y" = x.compose(y)

// Trivial implementation of permutation Group.
#[derive(Clone, Debug)]
pub struct Group {
    // all perms are bijections : omega -> omega.
    omega: HashSet<Slot>,
    next: Option<Box<Next>>,
}

#[derive(Clone, Debug)]
struct Next {
    // the Slot we are stabilizing
    stab: Slot,

    // the orbit tree.
    // ot[x] is a perm that maps stab to x.
    ot: HashMap<Slot, Perm>,

    g: Group,
}

impl Group {
    pub fn new(omega: &HashSet<Slot>, generators: HashSet<Perm>) -> Self {
        let omega = omega.clone();
        let next = find_lowest_nonstab(&generators)
                    .map(|s| Box::new(Next::new(s, &omega, generators)));
        Group { omega, next }
    }

    pub fn identity(omega: &HashSet<Slot>) -> Self {
        Self::new(omega, HashSet::default())
    }

    fn generators_impl(&self) -> HashSet<Perm> {
        match &self.next {
            None => HashSet::default(),
            Some(n) => &n.ot.values().cloned().collect::<HashSet<_>>() | &n.g.generators_impl(),
        }
    }

    pub fn generators(&self) -> HashSet<Perm> {
        let mut out = self.generators_impl();
        out.remove(&SlotMap::identity(&self.omega));
        out
    }

    // Should be very rarely called.
    pub fn all_perms(&self) -> HashSet<Perm> {
        match &self.next {
            None => [Perm::identity(&self.omega)].into_iter().collect(),
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

    pub fn contains(&self, p: &Perm) -> bool {
        match &self.next {
            None => p.iter().all(|(x, y)| x == y),
            Some(n) => {
                let Some(part) = &n.ot.get(&p[n.stab]) else { return false };
                n.g.contains(&p.compose(&part.inverse()))
            },
        }
    }

    pub fn add(&mut self, p: Perm) {
        self.add_set([p].into_iter().collect());
    }

    pub fn add_set(&mut self, mut perms: HashSet<Perm>) {
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

impl Next {
    fn new(stab: Slot, omega: &HashSet<Slot>, generators: HashSet<Perm>) -> Self {
        let ot = build_ot(stab, omega, &generators);
        let generators = schreiers_lemma(stab, &ot, generators);
        let g = Group::new(omega, generators);
        Next { stab, ot, g }
    }
}

fn build_ot(stab: Slot, omega: &HashSet<Slot>, generators: &HashSet<Perm>) -> HashMap<Slot, Perm> {
    let mut ot = HashMap::default();
    ot.insert(stab, SlotMap::identity(omega));

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
fn schreiers_lemma(stab: Slot, ot: &HashMap<Slot, Perm>, generators: HashSet<Perm>) -> HashSet<Perm> {
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
fn find_lowest_nonstab(generators: &HashSet<Perm>) -> Option<Slot> {
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
