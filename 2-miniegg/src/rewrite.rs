use crate::*;

// candidate for beta reduction.
struct Candidate {
    app: ENode,
    lam: ENode,
}

// applies rewrites (only beta-reduction) for all applicable situations.
pub fn rewrite_step(eg: &mut EGraph) {
    for cand in candidates(eg) {
        let app_id = eg.add(cand.app.clone()); // will not actually add, because it already exists.

        let ENode::App(_l, t) = cand.app.clone() else { panic!() };
        let ENode::Lam(x, b) = cand.lam.clone() else { panic!() };
        let id = subst(b, &x, t, eg);
        eg.union(id, app_id);
    }
}

fn candidates(eg: &EGraph) -> Vec<Candidate> {
    // find all lambdas:
    let mut lambdas: HashMap<Id, Vec<ENode>> = Default::default();
    for c in eg.ids() {
        let mut v = Vec::new();
        for enode in eg.enodes(c) {
            if let ENode::Lam(_, _) = &enode {
                v.push(enode.clone());
            }
        }

        lambdas.insert(c, v);
    }

    // find apps:
    let mut candidates = Vec::new();

    for c in eg.ids() {
        for enode in eg.enodes(c) {
            if let ENode::App(l, _t) = enode {
                for lam in lambdas[&l].clone() {
                    candidates.push(Candidate { app: enode.clone(), lam });
                }
            }
        }
    }

    candidates
}
