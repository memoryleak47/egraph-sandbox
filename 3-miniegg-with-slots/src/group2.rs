use crate::*;

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
}

impl Next {
    fn new(stab: Slot, omega: &HashSet<Slot>, generators: HashSet<Perm>) -> Self {
        let ot = build_ot(stab, omega, &generators);
        let generators = schreiers_lemma(generators);
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
            for (k, v) in ot.clone() {
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
fn schreiers_lemma(generators: HashSet<Perm>) -> HashSet<Perm> {
    todo!()
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
