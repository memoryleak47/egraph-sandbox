use crate::*;

use std::hash::{Hasher, Hash};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Equation {
    pub l: AppliedId,
    pub r: AppliedId,
}

#[derive(Clone, Debug)]
pub struct ExplicitProof(pub /*justification: */ Option<String>);
#[derive(Clone, Debug)]
pub struct ReflexivityProof;
#[derive(Clone, Debug)]
pub struct SymmetryProof(pub ProvenEq);
#[derive(Clone, Debug)]
pub struct TransitivityProof(pub ProvenEq, pub ProvenEq);
#[derive(Clone, Debug)]
pub struct CongruenceProof(pub Vec<ProvenEq>);

#[derive(Debug, Clone)]
pub enum Proof {
    Explicit(ExplicitProof),
    Reflexivity(ReflexivityProof),
    Symmetry(SymmetryProof),
    Transitivity(TransitivityProof),
    Congruence(CongruenceProof),

    // Both global renaming within equations and alpha-equivalence will be handled in the other rules too.
    // All equations will be understood as an arbitrary representative from its global renaming equivalence class.
    // So f(x, y) = g(x, y) is conceptually the same equation as f(a, b) = g(a, b).
    // In other words, whenever you use an equation, you always do it using "match_app_id".
}

pub type ProvenEq = Arc<ProvenEqRaw>;

#[derive(Debug, Clone)]
pub struct ProvenEqRaw {
    // fields are intentionally private so that only this module can construct instances for it.
    // These equations should always be fully "syn", i.e. they should not have any missing slot arguments, even redundant slots have to be passed explicitly.
    eq: Equation,
    proof: Proof,
}

impl PartialEq for ProvenEqRaw {
    // TODO normalize slotnames before this?
    fn eq(&self, other: &Self) -> bool { self.eq == other.eq }
}

impl Eq for ProvenEqRaw { }

impl Hash for ProvenEqRaw {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        // TODO normalize slotnames before this?
        self.eq.hash(hasher);
    }
}


impl ExplicitProof {
    pub fn check(&self, eq: &Equation) -> Option<ProvenEq> {
        assert(true)?;

        let eq = eq.clone();
        let proof = Proof::Explicit(self.clone());
        Some(Arc::new(ProvenEqRaw { eq, proof }))
    }
}

impl ReflexivityProof {
    pub fn check(&self, eq: &Equation) -> Option<ProvenEq>{
        assert(eq.l == eq.r)?;

        let eq = eq.clone();
        let proof = Proof::Reflexivity(self.clone());
        Some(Arc::new(ProvenEqRaw { eq, proof }))
    }
}

impl SymmetryProof {
    pub fn check(&self, eq: &Equation) -> Option<ProvenEq> {
        let SymmetryProof(x) = self;

        let flipped = Equation { l: x.r.clone(), r: x.l.clone() };
        match_equation(eq, &flipped).map(|_|())?;

        let eq = eq.clone();
        let proof = Proof::Symmetry(self.clone());
        Some(Arc::new(ProvenEqRaw { eq, proof }))
    }
}

impl TransitivityProof {
    pub fn check(&self, eq: &Equation) -> Option<ProvenEq> {
        let TransitivityProof(eq1, eq2) = self;

        let theta = match_app_id(&eq2.l, &eq1.r)?;
        let a = eq1.l.clone();
        let c = eq2.r.apply_slotmap_fresh(&theta);
        let out = Equation { l: a, r: c };
        match_equation(eq, &out).map(|_|())?;

        let eq = eq.clone();
        let proof = Proof::Transitivity(self.clone());
        Some(Arc::new(ProvenEqRaw { eq, proof }))
    }
}

impl CongruenceProof {
    pub fn check<L: Language>(&self, eq: &Equation, eg: &EGraph<L>) -> Option<ProvenEq> {
        let CongruenceProof(child_proofs) = self;

        let l = eg.get_syn_node(&eq.l);
        let r = eg.get_syn_node(&eq.r);

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

        let eq = eq.clone();
        let proof = Proof::Congruence(self.clone());
        Some(Arc::new(ProvenEqRaw { eq, proof }))
    }
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

impl<L: Language> EGraph<L> {
    pub fn get_syn_node(&self, i: &AppliedId) -> L {
        let syn = self.classes[&i.id].syn_enode.as_ref().unwrap();
        syn.apply_slotmap_fresh(&i.m)
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
