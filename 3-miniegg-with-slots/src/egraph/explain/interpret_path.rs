#![allow(unused)]

use crate::*;

// convert an EqPath (i.e. a proof-draft without congruence) to an explanation.
impl<L: Language> Explain<L> {
    pub fn interpret_path(&self, path: EqPath) -> Explanation<L> {
        let t = self.term_id_to_term(&path.start).unwrap();
        let init = Explanation { term: t, step: None };
        let mut out = vec![init];

        for eq in path.elems {
            out.push(self.interpret_equation(eq));
        }

        compose_explanation_list(out)
    }

    fn interpret_equation(&self, eq: Equation) -> Explanation<L> {
        let Equation { l, r, j } = eq;

        if Justification::Congruence == j {
            let mut x_enode = self.term_id_to_enode(&l).unwrap();
            let mut y_enode = self.term_id_to_enode(&r).unwrap();

            // make their inner private variables named the same:
            let xpriv = x_enode.private_slot_occurences_mut().into_iter();
            let ypriv = y_enode.private_slot_occurences_mut().into_iter();
            for (xp, yp) in xpriv.zip(ypriv) {
                *yp = *xp;
            }

            self.find_congruence_explanation(x_enode, y_enode)
        } else {
            let term_l = self.term_id_to_term(&l).unwrap();
            let term_r = self.term_id_to_term(&r).unwrap();
            Explanation {
                term: term_l,
                step: Some(Box::new(
                    ExplanationStep {
                        index_list: Vec::new(),
                        justification: j,
                        exp: Explanation {
                            term: term_r,
                            step: None,
                        },
                    },
                )),
            }
        }
    }

    fn find_congruence_explanation(&self, a: L, b: L) -> Explanation<L> {
        let l_a = a.applied_id_occurences();
        let l_b = b.applied_id_occurences();
        assert_eq!(l_a.len(), l_b.len());
        let n = l_a.len();

        let mut explanations = Vec::new();
        for i in 0..n {
            let c_a = &l_a[i];
            let c_b = &l_b[i];
            let base_expl = self.find_explanation(c_a, c_b);
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
