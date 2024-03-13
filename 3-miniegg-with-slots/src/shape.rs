use crate::*;

// an ENode that whose free (and redundant slots) have been renamed to the interval s0..sn,
// with lexicographically minimal naming.
pub type Shape = ENode;

impl ENode {
    // let n.shape() = (sh, bij); then
    // - sh.apply_slotmap(bij) is equivalent to n (excluding lambda variable renames)
    // - bij.slots() == n.slots(). Note that these would also include redundant slots.
    // - sh is the lexicographically lowest equivalent version of n, reachable by bijective renaming of slots (including redundant ones).
    pub fn shape(&self) -> (Shape, Bijection) {
        #[allow(non_snake_case)]
        let N: HashSet<Slot> = self.slots();

        match self.clone() {
            ENode::Var(s) => {
                let s0 = Slot(0);
                let sh = Shape::Var(s0);
                let bij = Bijection::from([(s0, s)]);

                (sh, bij)
            },
            ENode::Lam(s, x) => {
                let s0 = Slot(0);
                let mut bij = Bijection::new();
                bij.insert(s0, s);

                for sx in x.m.values_vec() {
                    if sx == s { continue; }

                    let next = Slot(bij.len());
                    bij.insert(next, sx);
                }

                let sh = Shape::Lam(s0, x.apply_slotmap(&bij.inverse()));
                bij.remove(s0);

                (sh, bij)
            },
            ENode::App(l, r) => {
                let mut bij = Bijection::new();

                for x in l.m.values_vec().into_iter().chain(r.m.values_vec().into_iter()) {
                    if !bij.contains_key(x) {
                        let next = Slot(bij.len());
                        bij.insert(x, next);
                    }
                }

                let sh = Shape::App(l.apply_slotmap(&bij), r.apply_slotmap(&bij));

                (sh, bij.inverse())
            },
        }
    }
}
