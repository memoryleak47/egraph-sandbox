use crate::*;

use std::sync::Arc;

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

    pub fn null() -> Arc<Self> {
        let app_id = AppliedId::null();
        let eq = Equation { l: app_id.clone(), r: app_id.clone() };
        let proof = Proof::Reflexivity;

        Arc::new(ProvenEq {
            eq,
            proof,
        })
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

            Proof::Reflexivity => assert(eq.l == eq.r),
            Proof::Symmetry(x) => {
                let x = x.eq().clone();
                let flipped = Equation { l: x.r, r: x.l };
                match_equation(eq, &flipped).map(|_|())
            }
            Proof::Transitivity(eq1, eq2) => {
                let eq1 = eq1.eq().clone();
                let eq2 = eq2.eq().clone();
                let theta = match_app_id(&eq2.l, &eq1.r)?;
                let a = eq1.l.clone();
                let c = eq2.r.apply_slotmap_fresh(&theta);
                let out = Equation { l: a, r: c };
                match_equation(eq, &out).map(|_|())
            },
            Proof::Congruence(child_proofs) => {
                let l = self.get_syn_node(&eq.l);
                let r = self.get_syn_node(&eq.r);
                todo!()
            }

            Proof::Shrink(witness) => {
                // witness: f(x, y) = c(x)
                // -> eq:   f(x, y) = f(x)
                assert(eq.l.id == eq.r.id)?;
                for (x, y) in eq.r.m.iter() {
                    assert(eq.l.m.get(x)? == y)?;
                }

                // The slots that are declared redundant by the "eq".
                // Note that we talk about the "values", not the "keys" here.
                // Thus these aren't public slots of the e-class "eq.l.id".
                let new_redundants = &eq.l.slots() - &eq.r.slots();

                let witness = witness.eq();
                let theta = match_app_id(&witness.l, &eq.l)?;
                let witness_r = witness.r.apply_slotmap_fresh(&theta);

                // Every slot that is named as redundant by the "eq", has to be missing in the r of the witness.
                let r_witness_slots = witness_r.slots();
                for x in new_redundants {
                    assert(!r_witness_slots.contains(&x))?;
                }
                assert(true)
            },
        }
    }

    fn get_syn_node(&self, i: &AppliedId) -> L {
        let syn = self.classes[&i.id].syn_enode.as_ref().unwrap();
        syn.apply_slotmap_fresh(&i.m)
    }

    fn get_sem_node(&self, i: &AppliedId) -> L {
        self.semify_enode(self.get_syn_node(i))
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
    let theta_l = match_app_id(&a.l, &b.l)?;
    let theta_r = match_app_id(&a.r, &b.r)?;

    let theta = theta_l.try_union(&theta_r)?;

    if CHECKS {
        assert_eq!(&a.apply_slotmap(&theta), b);
    }

    Some(theta)
}

fn apply_equation(x: &AppliedId, eq: &Equation) -> Option<AppliedId> {
    let theta = match_app_id(&eq.l, x)?;
    Some(eq.r.apply_slotmap_fresh(&theta))
}


fn assert(b: bool) -> Option<()> {
    if b { Some(()) }
    else { None }
}
