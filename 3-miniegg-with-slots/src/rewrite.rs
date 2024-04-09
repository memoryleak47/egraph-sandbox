use crate::*;

// candidate for beta reduction.
// Both ENodes are computed by "sh.apply_slotmap(bij)", where (sh, bij) in EClass::nodes from their respective classes.
pub struct Candidate {
    pub app: ENode,
    pub lam: ENode,
}

// applies rewrites (only beta-reduction) for all applicable situations.
pub fn rewrite_step(eg: &mut EGraph<ENode>) {
    for cand in candidates(eg) {
        let app_id = eg.lookup(&cand.app).unwrap();

        let ENode::App(l, t) = cand.app.clone() else { panic!() };
        let ENode::Lam(x, b) = cand.lam.clone() else { panic!() };
        assert_eq!(x, Slot(0));

        // l.m :: slots(lam) -> slots(app)
        let mut m = l.m.clone();

        // if x is a public slot of "app", we'd need to rename. But as x should always be s0 this shouldn't come up.
        assert!(!m.contains_key(x));

        m.insert(x, x);

        let b = b.apply_slotmap(&m);

        let new_id = subst(b, x, t, eg);
        eg.union(new_id, app_id);
    }
}

pub fn candidates(eg: &EGraph<ENode>) -> Vec<Candidate> {
    // find all lambdas:
    let mut lambdas: HashMap<Id, Vec<ENode>> = Default::default();
    for c in eg.ids() {
        let mut v = Vec::new();
        for enode in eg.enodes(c) {
            if matches!(enode, ENode::Lam(..)) {
                v.push(enode.clone());
            }
        }

        lambdas.insert(c, v);
    }

    // find apps:
    let mut candidates = Vec::new();

    for c in eg.ids() {
        for enode in eg.enodes(c) {
            if let ENode::App(l, _t) = &enode {
                for lam in lambdas[&l.id].clone() {
                    candidates.push(Candidate { app: enode.clone(), lam });
                }
            }
        }
    }

    candidates
}
