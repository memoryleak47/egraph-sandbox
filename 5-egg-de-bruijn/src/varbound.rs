use crate::*;

// Each EClass stores a u32.
// If the EClass has no free variables, this u32 is 0.
// Let x be the largest de bruijn index free variable contained in the EClass, then the EClass stores x+1.
#[derive(Default)]
pub struct Varbound;

impl Analysis<ENode> for Varbound {
    type Data = u32;

    fn make(eg: &EGraph<ENode, Self>, enode: &ENode) -> Self::Data {
        let d = |i: &Id| eg[*i].data;
        match enode {
            ENode::Var(x) => x+1,
            ENode::App([l, r]) => d(l).max(d(r)),
            ENode::Lam(b) => d(b).max(1)-1, // The ".max(1)" is necessary for eg. \x. \y. y
            ENode::Placeholder(_) => 0,
        }
    }

    fn merge(&mut self, a: &mut Self::Data, b: Self::Data) -> DidMerge {
        merge_max(a, b)
    }
}
