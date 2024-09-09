#![allow(unused)]

use crate::*;

// The pre-processing step in the E-Graph that is ran before asking the Explain-module for the explanations.

impl<L: Language> EGraph<L> {
    pub fn explain_equivalence(&mut self, a: RecExpr<L>, b: RecExpr<L>) -> Explanation<L> {
        let a_ = self.add_expr(a.clone());
        let b_ = self.add_expr(b.clone());
        assert!(self.eq(&a_, &b_));
        let Some(explain) = self.explain.as_mut() else { panic!() };
        let a_expl = explain.add_term(&a);
        let b_expl = explain.add_term(&b);
        let _ = explain;

        self.add_congruence_equations();

        let Some(explain) = self.explain.as_mut() else { panic!() };
        explain.compute_incidence_map();

        let out = explain.find_explanation(&a_expl, &b_expl);
        
        explain.imap.clear();
        self.remove_congruence_equations();

        out
    }

    fn add_congruence_equations(&mut self) {
        let back_translator = self.generate_back_translator();
        let back_translate = |n: &L| n.map_applied_ids(|child| get_applied(&back_translator, &child).unwrap());

        let Some(explain) = self.explain.as_ref() else { panic!() };
        let mut eqs = Vec::new();

        // maps a strong-shape of a child-wise normalized egraph e-node to a explain applied id corresponding to it.
        let mut shapes_map: HashMap<L, AppliedId> = HashMap::default();

        for (i, n) in &explain.term_id_to_enode {
            let i = explain.mk_identity_app_id(*i);

            let n2 = back_translate(n);
            let (sh, bij) = self.shape(&n2);
            if let Some(orig) = shapes_map.get(&sh) {
                let orig = orig.apply_slotmap(&bij);
                eqs.push((orig, i, Justification::Congruence));
            } else {
                shapes_map.insert(sh, i.apply_slotmap(&bij.inverse()));
            }
        }

        let Some(explain) = self.explain.as_mut() else { panic!() };
        for (a, b, j) in eqs {
            explain.add_equation(a, b, j);
        }
    }

    // for each Explain Id, it finds the normal form e-graph AppliedId.
    fn generate_back_translator(&self) -> HashMap<Id, AppliedId> {
        let Some(explain) = self.explain.as_ref() else { panic!() };

        let mut bt = HashMap::default();

        for (i, _) in &explain.term_id_to_enode {
            let i = explain.mk_identity_app_id(*i);
            let term = explain.term_id_to_term(&i).unwrap();
            let orig = lookup_rec_expr(&term, self).unwrap();
            insert_applied(&mut bt, i, orig);
        }

        bt
    }

    fn remove_congruence_equations(&mut self) {
        let Some(explain) = self.explain.as_mut() else { panic!() };
        explain.equations.retain(|eq| !matches!(eq.j, Justification::Congruence));
    }
}

impl<L: Language> Explain<L> {
    fn compute_incidence_map(&mut self) {
        self.imap.clear();

        for (&i, _) in &self.term_id_to_enode {
            self.imap.insert(i, HashSet::default());
        }

        for (i, Equation { l, r, .. }) in self.equations.iter().enumerate() {
            self.imap.get_mut(&l.id).unwrap().insert(i);
            self.imap.get_mut(&r.id).unwrap().insert(i);
        }
    }
}

fn insert_applied(map: &mut HashMap<Id, AppliedId>, k: AppliedId, v: AppliedId) {
    // map[k] == v
    // map[k.id * k.m] == v
    // map[k.id] == v * k.m^-1
    map.insert(k.id, v.apply_slotmap(&k.m.inverse()));
}

fn get_applied(map: &HashMap<Id, AppliedId>, k: &AppliedId) -> Option<AppliedId> {
    // map[k] == v
    // map[k.id * k.m] == v
    // map[k.id] == v * k.m^-1
    // map[k.id] * k.m == v
    map.get(&k.id).map(|x| x.apply_slotmap(&k.m.inverse()))
}
