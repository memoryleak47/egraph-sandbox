use crate::*;
    
// an ENode that has been reduced to its shape.
pub type Shape = ENode;

impl EGraph {
    // let eg.shape(n) = (x, y); then
    // - x.apply_slotmap(y) is equivalent to n (excluding lambda variable renames)
    // - y.slots() == n.slots(). Note that these would also include redundant slots.
    // - x is the lexicographically lowest equivalent version of n, reachable by bijective renaming of slots and re-ordering of AppliedId-args.
    // - Note that y is not normalized! There are multiple possible outputs for y, depending on the symmetries of the EClass containing this shape.
    //
    // For two ENodes n1, n2 that only differentiate each other by
    // (1) the names of their public slots (including redundant slots) and lambda slots, and
    // (2) the order of their AppliedId arguments within the boundaries of their corresponding permutation groups;
    // then self.shape(n1).0 == self.shape(n2).0
    pub fn shape(&self, n: &ENode) -> (Shape, Bijection) {
        let n = self.normalize_enode_by_unionfind(n);
        match n {
            ENode::Var(s) => {
                let s0 = Slot(0);
                let l = Shape::Var(s0);
                let r = Bijection::from([(s0, s)]);

                (l, r)
            },
            ENode::Lam(s, x) => {
                let s0 = Slot(0);
                let mut r = Bijection::new();
                r.insert(s0, s);

                for sx in x.m.values_vec() {
                    let next = Slot(r.len());
                    r.insert(next, sx);
                }

                let l = Shape::Lam(s0, x.apply_slotmap(&r.inverse()));
                r.remove(s0);

                (l, r)
            },
            ENode::App(l, r) => {
                todo!()
            },
        }
    }
}
