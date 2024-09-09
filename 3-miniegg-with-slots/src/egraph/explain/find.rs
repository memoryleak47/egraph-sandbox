#![allow(unused)]

use crate::*;

impl<L: Language> Explain<L> {
    pub fn find_explanation(&self, a: &AppliedId, b: &AppliedId, imap: &IMap) -> Explanation<L> {
        let expl = self.find_explanation_modulo_slots(a, b.id, imap);

        let final_term = self.term_id_to_term(b).unwrap();

        if !alpha_eq(expl.last(), &final_term) {
            panic!("Slot mismatch! Explanations don't yet work for redundant slots & symmetries");
        }

        expl
    }

    fn find_explanation_modulo_slots(&self, a: &AppliedId, b_id: Id, imap: &IMap) -> Explanation<L> {
        if a.id == b_id {
            let t = self.term_id_to_term(a).unwrap();
            return Explanation {
                term: t,
                step: None
            };
        }

        // maps each Id `r_id` to an `Equation(l, r, j)`,
        // where r_id = r.id and
        // l.id is a step closer to a.id.
        let mut pred: HashMap<Id, Equation> = HashMap::default();

        let mut open = HashSet::default();
        open.insert(a.id);

        while open.len() > 0 {
            let last_open = open;
            open = HashSet::default();

            for x in last_open {
                for &i in &imap[&x] {
                    let mut eq = self.equations[i].clone();

                    // flip x to be on the left-side of the equation.
                    if x != eq.l.id {
                        eq = eq.flip();
                    }
                    let l = eq.l.id;
                    let r = eq.r.id;
                    assert_eq!(x, l);

                    if !pred.contains_key(&r) && r != a.id {
                        pred.insert(r, eq);
                        open.insert(r);
                    }
                }
            }
        }

        assert!(pred.contains_key(&b_id));

        // path b -> a
        let mut path = vec![b_id];
        let mut i = b_id;
        while i != a.id {
            i = pred[&i].l.id;
            path.push(i);
        }

        // path a -> b
        path.reverse();

        return rec(self, &path[..], &pred, imap);

        fn rec<L: Language>(explain: &Explain<L>, path: &[Id], pred: &HashMap<Id, Equation>, imap: &IMap) -> Explanation<L> {
            let x = path[0];

            let app_id_x = explain.mk_identity_app_id(x);
            let term_x = explain.term_id_to_term(&app_id_x).unwrap();

            if path.len() == 1 {
                return Explanation { term: term_x, step: None };
            }

            let y = path[1];
            let app_id_y = explain.mk_identity_app_id(y);
            let term_y = explain.term_id_to_term(&app_id_y).unwrap();

            let j = pred[&y].j.clone();

            let explanation_step = if Justification::Congruence == j {
                let x_enode = explain.term_id_to_enode(&app_id_x).unwrap();
                let y_enode = explain.term_id_to_enode(&app_id_y).unwrap();
                explain.find_congruence_explanation(x_enode, y_enode, imap)
            } else {
                Explanation {
                    term: term_x,
                    step: Some(Box::new(
                        ExplanationStep {
                            index_list: Vec::new(),
                            justification: j,
                            exp: Explanation {
                                term: term_y,
                                step: None,
                            },
                        },
                    )),
                }
            };

            let tail = rec(explain, &path[1..], pred, imap);
            compose_explanation(explanation_step, tail)
        }
    }

    fn find_congruence_explanation(&self, a: L, b: L, imap: &IMap) -> Explanation<L> {
        let l_a = a.applied_id_occurences();
        let l_b = b.applied_id_occurences();
        assert_eq!(l_a.len(), l_b.len());
        let n = l_a.len();

        let mut explanations = Vec::new();
        for i in 0..n {
            let c_a = &l_a[i];
            let c_b = &l_b[i];
            let base_expl = self.find_explanation(c_a, c_b, imap);
            let lifted = lift(base_expl, i, self, &a, &b, &l_a, &l_b);

            explanations.push(lifted);

            fn lift<L: Language>(exp: Explanation<L>, i: usize, explain: &Explain<L>, a: &L, b: &L, l_a: &[AppliedId], l_b: &[AppliedId]) -> Explanation<L> {
                Explanation {
                    term: lift_term(exp.term, i, explain, a, b, l_a, l_b),
                    step: exp.step.map(|step| {
                        let mut index_list = step.index_list;
                        index_list.insert(0, i);
                        Box::new(ExplanationStep {
                            index_list,
                            justification: step.justification,
                            exp: lift(step.exp, i, explain, a, b, l_a, l_b),
                        })
                    }),
                }
            }

            fn lift_term<L: Language>(t: RecExpr<L>, i: usize, explain: &Explain<L>, a: &L, b: &L, l_a: &[AppliedId], l_b: &[AppliedId]) -> RecExpr<L> {
                let n = l_a.len();
                let node = nullify_app_ids(a);

                let mut children = Vec::new();
                for j in 0..i {
                    children.push(explain.term_id_to_term(&l_b[j]).unwrap());
                }
                children.push(t);
                for j in (i+1)..n {
                    children.push(explain.term_id_to_term(&l_a[j]).unwrap());
                }

                RecExpr {
                    node,
                    children,
                }
            }
        }

        compose_explanation_list(explanations)
    }
}
