use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Equation {
    pub l: AppliedId,
    pub r: AppliedId,
}

impl Equation {
    pub fn apply_slotmap(&self, m: &SlotMap) -> Self {
        Equation {
            l: self.l.apply_slotmap(&m),
            r: self.r.apply_slotmap(&m),
        }
    }

    pub fn apply_slotmap_fresh(&self, m: &SlotMap) -> Self {
        Equation {
            l: self.l.apply_slotmap_fresh(&m),
            r: self.r.apply_slotmap_fresh(&m),
        }
    }
}

pub type ProvenEq = Arc<ProvenEqRaw>;

#[derive(Debug, Clone)]
pub struct ProvenEqRaw {
    // fields are intentionally private so that only "add_proof" can construct instances for it.
    eq: Equation,
    proof: Proof,
}

impl Deref for ProvenEqRaw {
    type Target = Equation;

    fn deref(&self) -> &Equation {
        &self.eq
    }
}

impl ProvenEqRaw {
    pub fn proof(&self) -> &Proof {
        &self.proof
    }
}

#[derive(Debug, Clone)]
pub enum Proof {
    Explicit(/*justification: */ Option<String>),

    Reflexivity,
    Symmetry(ProvenEq),
    Transitivity(ProvenEq, ProvenEq),
    Congruence(Vec<ProvenEq>),

    // Both global renaming within equations and alpha-equivalence will be handled in the other rules too.
    // All equations will be understood as an arbitrary representative from its global renaming equivalence class.
    // So f(x, y) = g(x, y) is conceptually the same equation as f(a, b) = g(a, b).
    // In other words, whenever you use an equation, you always do it using "match_app_id".
}

impl<L: Language> EGraph<L> {
    pub fn prove(&self, eq: Equation, proof: Proof) -> Option<ProvenEq> {
        self.check_proof(&eq, &proof)?;

        Some(Arc::new(ProvenEqRaw { eq, proof }))
    }

    pub fn check_proof(&self, eq: &Equation, proof: &Proof) -> Option<()> {
        match proof {
            Proof::Explicit(_) => assert(true),

            Proof::Reflexivity => assert(eq.l == eq.r),
            Proof::Symmetry(x) => {
                let flipped = Equation { l: x.r.clone(), r: x.l.clone() };
                match_equation(eq, &flipped).map(|_|())
            }
            Proof::Transitivity(eq1, eq2) => {
                let theta = match_app_id(&eq2.l, &eq1.r)?;
                let a = eq1.l.clone();
                let c = eq2.r.apply_slotmap_fresh(&theta);
                let out = Equation { l: a, r: c };
                match_equation(eq, &out).map(|_|())
            },
            Proof::Congruence(child_proofs) => {
                let l = self.get_syn_node(&eq.l);
                let r = self.get_syn_node(&eq.r);

                let null_l = nullify_app_ids(&l);
                let null_r = nullify_app_ids(&r);
                assert(null_l == null_r)?;

                let l_v = l.applied_id_occurences();
                let r_v = r.applied_id_occurences();

                assert(l_v.len() == child_proofs.len())?;
                assert(r_v.len() == child_proofs.len())?;

                let l_v = l_v.into_iter();
                let r_v = r_v.into_iter();

                let c_v = child_proofs.into_iter();
                for ((ll, rr), prf) in l_v.zip(r_v).zip(c_v) {
                    let eq1 = &Equation { l: ll, r: rr };
                    let eq2 = prf.deref();
                    match_equation(eq1, eq2)?;
                }
                assert(true)
            }
        }
    }

    pub fn get_syn_node(&self, i: &AppliedId) -> L {
        let syn = self.classes[&i.id].syn_enode.as_ref().unwrap();
        syn.apply_slotmap_fresh(&i.m)
    }

    fn get_sem_node(&self, i: &AppliedId) -> L {
        self.semify_enode(self.get_syn_node(i))
    }
}

// returns the global renaming theta, s.t. a.apply_slotmap(theta) = b, if it exists.
pub fn match_app_id(a: &AppliedId, b: &AppliedId) -> Option<SlotMap> {
    assert(a.id == b.id)?;
    assert(a.m.keys() == b.m.keys())?;

    // a.m :: slots(i) -> A
    // b.m :: slots(i) -> B
    // theta :: A -> B
    let theta = a.m.inverse().compose(&b.m);

    if CHECKS {
        assert_eq!(&a.apply_slotmap(&theta), b);
    }

    Some(theta)
}

// returns the bijective renaming theta, s.t. a.apply_slotmap(theta) = b, if it exists.
pub fn match_equation(a: &Equation, b: &Equation) -> Option<SlotMap> {
    let theta_l = match_app_id(&a.l, &b.l)?;
    let theta_r = match_app_id(&a.r, &b.r)?;

    let theta = theta_l.try_union(&theta_r)?;
    assert(theta.is_bijection())?;

    if CHECKS {
        assert_eq!(&a.apply_slotmap(&theta), b);
    }

    Some(theta)
}

pub fn apply_equation(x: &AppliedId, eq: &Equation) -> Option<AppliedId> {
    let theta = match_app_id(&eq.l, x)?;
    Some(eq.r.apply_slotmap_fresh(&theta))
}


pub fn assert(b: bool) -> Option<()> {
    if b { Some(()) }
    else { None }
}
