use crate::*;

#[track_caller]
pub fn prove_explicit(l: &AppliedId, r: &AppliedId, j: Option<String>) -> ProvenEq {
    let eq = Equation { l: l.clone(), r: r.clone() };
    ExplicitProof(j).check(&eq)
}

#[track_caller]
pub fn prove_reflexivity(id: &AppliedId) -> ProvenEq {
    let eq = Equation { l: id.clone(), r: id.clone() };
    ReflexivityProof.check(&eq)
}

#[track_caller]
pub fn prove_symmetry(x: ProvenEq) -> ProvenEq {
    let eq = Equation { l: x.r.clone(), r: x.l.clone() };
    SymmetryProof(x).check(&eq)
}

#[track_caller]
pub fn prove_transitivity(x: ProvenEq, y: ProvenEq) -> ProvenEq {
    let eq1 = x.clone();
    let eq2 = y.clone();
    let theta = match_app_id(&eq2.l, &eq1.r);
    let a = eq1.l.clone();
    let c = eq2.r.apply_slotmap_fresh(&theta);
    let eq = Equation { l: a, r: c };

    TransitivityProof(x.clone(), y.clone()).check(&eq)
}

#[track_caller]
pub fn prove_congruence<L: Language>(l: &AppliedId, r: &AppliedId, child_proofs: Vec<ProvenEq>, eg: &EGraph<L>) -> ProvenEq {
    // because all our proofs witness redundancy per default, we need to sometimes get rid of these redundancies in child-proofs
    // eg. when proving reflexivity e-nodes like a[x,y] + b[y,z] = a[x,y] + b[y,z].
    // and we only have equations like a[x,y] = a[x',y'] witnessing redundancies.

    let l_syn_node = alpha_normalize(&eg.get_syn_node(l));
    let r_syn_node = alpha_normalize(&eg.get_syn_node(r));

    let l_ids = l_syn_node.applied_id_occurences().into_iter();
    let r_ids = r_syn_node.applied_id_occurences().into_iter();
    let child_proofs = child_proofs.into_iter();

    let mut new_child_proofs = Vec::new();
    for ((li, ri), prf) in l_ids.zip(r_ids).zip(child_proofs) {
        let goal = Equation { l: li, r: ri };
        let new_proof = eg.associate_necessaries(&goal, prf);
        new_child_proofs.push(new_proof);
    }

    let eq = Equation { l: l.clone(), r: r.clone() };
    CongruenceProof(new_child_proofs).check(&eq, eg)
}


impl<L: Language> EGraph<L> {
    fn semify_equation(&self, eq: &Equation) -> Equation {
        Equation {
            l: self.semify_app_id(eq.l.clone()),
            r: self.semify_app_id(eq.r.clone()),
        }
    }

    // peq seems to imply `goal` if we remove all the redundant slots.
    // now we want a proof that even makes the redundant slots work out.
    // We assume that `peq` is already maximally dis-associated. Hence we only need to re-associate some redundant slots to reach the goal.
    fn associate_necessaries(&self, goal: &Equation, peq: ProvenEq) -> ProvenEq {
        if CHECKS {
            assert_match_equation(&self.semify_equation(goal), &self.semify_equation(&peq));
        }
        let l_red = self.get_redundancy_proof(peq.l.id);

        // goal.l.m :: slots(goal.l.id) -> X
        // goal.r.m :: slots(goal.r.id) -> X
        // goal_associations :: slots(goal.l.id) -> slots(goal.r.id)
        let goal_associations = goal.l.m.compose_partial(&goal.r.m.inverse());

        let mut current = peq;
        let current_associations = current.l.m.compose_partial(&current.r.m.inverse());
        let open_association_keys = &goal_associations.keys() - &current_associations.keys();

        let l_red_slots = &self.syn_slots(current.l.id) - &self.slots(current.l.id);
        let intersection = &open_association_keys & &l_red_slots;
        if intersection.len() > 0 {
            let mut subgoal = current.equ();
            for x in intersection {
                let f = Slot::fresh();
                subgoal.l.m.insert(x, f);
                subgoal.r.m.insert(goal_associations[x], f);
            }
            current = TransitivityProof(l_red, current).check(&subgoal);
        }

        let r_red = self.get_redundancy_proof(current.r.id);
        TransitivityProof(current, r_red).check(goal)
    }

    fn disassociation_necessary(&self, peq: &ProvenEq) -> bool {
        let l_rev = peq.l.m.inverse();
        let r_rev = peq.r.m.inverse();
        let l_slots = self.slots(peq.l.id);
        let r_slots = self.slots(peq.r.id);
        for s in &peq.l.slots() & &peq.r.slots() {
            if !l_slots.contains(&l_rev[s]) { return true; }
            if !r_slots.contains(&r_rev[s]) { return true; }
        }

        false
    }

    fn get_redundancy_proof(&self, i: Id) -> ProvenEq {
        let (leader, prf) = self.proven_unionfind_get(i);
        let red_prf = self.classes[&leader.id].redundancy_proof.clone();
        let inv_prf = prove_symmetry(prf.clone());
        let out = prove_transitivity(prf, prove_transitivity(red_prf, inv_prf));
        out
    }

    pub fn disassociate_proven_eq(&self, peq: ProvenEq) -> ProvenEq {
        if self.disassociation_necessary(&peq) {
            let mut peq = peq;
            let x = self.get_redundancy_proof(peq.l.id);
            let y = self.get_redundancy_proof(peq.r.id);
            peq = prove_transitivity(x, peq);
            peq = prove_transitivity(peq, y);

            peq
        } else {
            peq
        }
    }
}

// This API should be ignoring the values of redundant slots.
// This means that whether you pre-randomize all the Slots mapped to redundant Slots in both goal&input-proofs before passing them to prove_X should not affect the outcome.
// Further it should always produce maximally disassociated output.
impl<L: Language> EGraph<L> {
    #[track_caller]
    pub fn prove_explicit(&self, l: &AppliedId, r: &AppliedId, j: Option<String>) -> ProvenEq {
        self.check_syn_applied_id(l);
        self.check_syn_applied_id(r);
        self.disassociate_proven_eq(prove_explicit(l, r, j))
    }

    #[track_caller]
    pub fn prove_reflexivity(&self, id: &AppliedId) -> ProvenEq {
        self.check_syn_applied_id(id);
        self.disassociate_proven_eq(prove_reflexivity(id))
    }

    #[track_caller]
    pub fn prove_symmetry(&self, x: ProvenEq) -> ProvenEq {
        self.disassociate_proven_eq(prove_symmetry(x))
    }

    #[track_caller]
    pub fn prove_transitivity(&self, x: ProvenEq, y: ProvenEq) -> ProvenEq {
        self.disassociate_proven_eq(prove_transitivity(x, y))
    }

    #[track_caller]
    pub fn prove_congruence(&self, l: &AppliedId, r: &AppliedId, child_proofs: Vec<ProvenEq>) -> ProvenEq {
        self.check_syn_applied_id(l);
        self.check_syn_applied_id(r);
        self.disassociate_proven_eq(prove_congruence(l, r, child_proofs, self))
    }
}
