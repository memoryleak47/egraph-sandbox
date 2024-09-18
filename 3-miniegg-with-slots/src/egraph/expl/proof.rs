use crate::*;

use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Equation {
    pub lhs: AppliedId,
    pub rhs: AppliedId,
}

impl Equation {
    pub fn apply_slotmap(&self, m: &SlotMap) -> Self {
        Equation {
            lhs: self.lhs.apply_slotmap(&m),
            rhs: self.rhs.apply_slotmap(&m),
        }
    }

    pub fn apply_slotmap_fresh(&self, m: &SlotMap) -> Self {
        Equation {
            lhs: self.lhs.apply_slotmap_fresh(&m),
            rhs: self.rhs.apply_slotmap_fresh(&m),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProvenEq {
    // fields are intentionally private so that only "add_proof" can construct instances for it.
    eq: Equation,
    proof: Proof,
}

impl ProvenEq {
    pub fn eq(&self) -> &Equation {
        &self.eq
    }

    pub fn proof(&self) -> &Proof {
        &self.proof
    }
}

#[derive(Debug, Clone)]
pub enum Proof {
    Explicit(/*justification: */ Option<String>),

    Reflexivity,
    Symmetry(Arc<ProvenEq>),
    Transitivity(Arc<ProvenEq>, Arc<ProvenEq>),
    Congruence(Vec<Arc<ProvenEq>>),

    Shrink(/*an equation witnessing redundant slots*/ Arc<ProvenEq>),

    // Both global renaming within equations and alpha-equivalence will be handled in the other rules too.
    // All equations will be understood as an arbitrary representative from its global renaming equivalence class.
    // So f(x, y) = g(x, y) is conceptually the same equation as f(a, b) = g(a, b).
    // In other words, whenever you use an equation, you always do it using "match_app_id".
}

impl<L: Language> EGraph<L> {
    pub fn prove(&self, eq: Equation, proof: Proof) -> Option<Arc<ProvenEq>> {
        self.check_proof(&eq, &proof)?;

        Some(Arc::new(ProvenEq { eq, proof }))
    }

    pub fn check_proof(&self, eq: &Equation, proof: &Proof) -> Option<()> {
        match proof {
            Proof::Explicit(_) => assert(true),

            Proof::Reflexivity => assert(eq.lhs == eq.rhs),
            Proof::Symmetry(x) => {
                let x = x.eq().clone();
                let flipped = Equation { lhs: x.rhs, rhs: x.lhs };
                match_equation(eq, &flipped).map(|_|())
            }
            Proof::Transitivity(eq1, eq2) => {
                let eq1 = eq1.eq().clone();
                let eq2 = eq2.eq().clone();
                let theta = match_app_id(&eq2.lhs, &eq1.rhs)?;
                let a = eq1.lhs.clone();
                let c = eq2.rhs.apply_slotmap_fresh(&theta);
                let out = Equation { lhs: a, rhs: c };
                match_equation(eq, &out).map(|_|())
            },
            Proof::Congruence(_child_proofs) => {
                todo!()
            },

            Proof::Shrink(witness) => {
                // witness: f(x, y) = c(x)
                // -> eq:   f(x, y) = f(x)
                assert(eq.lhs.id == eq.rhs.id)?;
                for (x, y) in eq.rhs.m.iter() {
                    assert(eq.lhs.m.get(x)? == y)?;
                }

                // The slots that are declared redundant by the "eq".
                // Note that we talk about the "values", not the "keys" here.
                // Thus these aren't public slots of the e-class "eq.lhs.id".
                let new_redundants = &eq.lhs.slots() - &eq.rhs.slots();

                let witness = witness.eq();
                let theta = match_app_id(&witness.lhs, &eq.lhs)?;
                let witness_rhs = witness.rhs.apply_slotmap_fresh(&theta);

                // Every slot that is named as redundant by the "eq", has to be missing in the rhs of the witness.
                let rhs_witness_slots = witness_rhs.slots();
                for x in new_redundants {
                    assert(!rhs_witness_slots.contains(&x))?;
                }
                assert(true)
            },
        }
    }
}

// returns the global renaming theta, s.t. a.apply_slotmap(theta) = b, if it exists.
fn match_app_id(a: &AppliedId, b: &AppliedId) -> Option<SlotMap> {
    assert(a.id == b.id)?;
    assert(a.m.keys() == b.m.keys())?;

    let theta = b.m.compose(&a.m.inverse());

    if CHECKS {
        assert_eq!(&a.apply_slotmap(&theta), b);
    }

    Some(theta)
}

// returns the global renaming theta, s.t. a.apply_slotmap(theta) = b, if it exists.
fn match_equation(a: &Equation, b: &Equation) -> Option<SlotMap> {
    let theta_l = match_app_id(&a.lhs, &b.lhs)?;
    let theta_r = match_app_id(&a.rhs, &b.rhs)?;

    let theta = theta_l.try_union(&theta_r)?;

    if CHECKS {
        assert_eq!(&a.apply_slotmap(&theta), b);
    }

    Some(theta)
}

fn apply_equation(x: &AppliedId, eq: &Equation) -> Option<AppliedId> {
    let theta = match_app_id(&eq.lhs, x)?;
    Some(eq.rhs.apply_slotmap_fresh(&theta))
}


fn assert(b: bool) -> Option<()> {
    if b { Some(()) }
    else { None }
}
