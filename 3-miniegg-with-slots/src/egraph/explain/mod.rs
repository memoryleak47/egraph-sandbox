#![allow(unused)]

use crate::*;

mod pre;
pub use pre::*;

mod explanation;
pub use explanation::*;

mod alpha;
pub use alpha::*;

// In the context of explanations, there is a bijection between Ids and Terms.
// Hence Ids uniquely identify certain concrete terms.

type EquationId = usize;
type IMap = HashMap<Id, HashSet<EquationId>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Equation {
    pub l: AppliedId,
    pub r: AppliedId,
    pub j: Justification,
}

impl Equation {
    fn flip(&self) -> Self {
        Equation {
            l: self.r.clone(),
            r: self.l.clone(),
            j: self.j.clone(),
        }
    }
}

// Invariants:
// - each Id from the egraph (dead or alive) has an associated e-node in term_id_to_enode.
#[derive(Debug)]
pub struct Explain<L: Language> {
    // translates E-Graph Ids into Term Ids.
    // This contains slot-name choices for redundant slots (because the term-world doesn't have redundant slots).
    // These choices are fixed, and are never renamed / refreshed.
    pub translator: HashMap<Id, AppliedId>,

    // These two form a bijection:
    pub enode_to_term_id: HashMap<L/*shape*/, AppliedId>,
    pub term_id_to_enode: HashMap<Id, L/*with identity perm*/>,

    pub equations: Vec<Equation>,
}

impl<L: Language> Default for Explain<L> {
    fn default() -> Self {
        Self {
            translator: Default::default(),
            enode_to_term_id: Default::default(),
            term_id_to_enode: Default::default(),
            equations: Default::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Justification {
    Congruence,
    Rule(String),
    Explicit, // union called without a rule.
}

impl<L: Language> Explain<L> {
    // translates an egraph e-class to its corresponding term id.
    pub fn translate(&self, l: &AppliedId) -> AppliedId {
        // l.m :: slots(l.id) -> X
        let a = &self.translator[&l.id];
        // a == l.id

        // a has some redundant slot choices.
        let mut m = l.m.clone();
        for s in a.slots() {
            if !m.contains_key(s) {
                m.insert(s, Slot::fresh());
            }
        }
        a.apply_slotmap(&m)
    }

    // translates an egraph e-node to its corresponding explain e-node.
    pub fn translate_enode(&self, e: &L) -> L {
        e.map_applied_ids(|x| self.translate(&x))
    }

    // Both l and i are in egraph world.
    pub fn add_translation(&mut self, l: L, i: AppliedId) {
        // l == i holds in the egraph world.
        let i2 = self.add_egraph_enode(l);
        // i should now translate to i2.
        
        // i == i2
        // i.id * i.m == i2
        // i.id == i2 * i.m^-1
        let i2_id = i2.apply_slotmap(&i.m.inverse());
        self.translator.insert(i.id, i2_id);
    }
 
    pub fn add_egraph_enode(&mut self, l: L) -> AppliedId {
        let l = self.translate_enode(&l);
        self.add_explain_enode(l)
    }

    // adds an e-node to the term-id <-> e-node bijection.
    // and returns the corresponding AppliedId.
    // Both input and output are completely in the explain-world.
    pub fn add_explain_enode(&mut self, l: L) -> AppliedId {
        let (sh, bij) = l.weak_shape();
        if let Some(x) = self.enode_to_term_id.get(&sh) {
            x.apply_slotmap(&bij)
        } else {
            let i = Id(self.enode_to_term_id.len());
            // i == l
            // -> i == sh * bij
            // -> sh == i * bij^-1
            let app_id = AppliedId::new(i, bij.inverse());
            self.enode_to_term_id.insert(sh, app_id);
            self.term_id_to_enode.insert(i, l.clone());
            let identity = bij.inverse().compose(&bij);
            AppliedId::new(i, identity)
        }
    }

    pub fn add_term(&mut self, t: &RecExpr<L>) -> AppliedId {
        let mut n = t.node.clone();
        let mut refs: Vec<&mut _> = n.applied_id_occurences_mut();
        for i in 0..refs.len() {
            *(refs[i]) = self.add_term(&t.children[i]);
        }
        self.add_explain_enode(n)
    }

    pub fn enode_to_term_id(&self, l: &L) -> Option<AppliedId> {
        let (sh, bij) = l.weak_shape();
        let a = self.enode_to_term_id.get(&sh)?;
        // a == sh by definition of a.
        // sh * bij == l by definition of (sh, bij).
        // -> a * bij == l
        Some(a.apply_slotmap(&bij))
    }

    pub fn term_id_to_enode(&self, a: &AppliedId) -> Option<L> {
        let x = self.term_id_to_enode.get(&a.id)?;
        // x == a.id by definition of x.
        // a == a.id * a.m by definition of AppliedId.
        // -> a == x * a.m
        let out = x.apply_slotmap(&a.m);
        let out = out.refresh_internals(out.slots());
        Some(out)
    }

    pub fn term_id_to_term(&self, a: &AppliedId) -> Option<RecExpr<L>> {
        let enode = self.term_id_to_enode(a)?;
        let cs = enode.applied_id_occurences()
                      .iter()
                      .map(|x| self.term_id_to_term(x).unwrap())
                      .collect();
        Some(RecExpr {
            node: nullify_app_ids(&enode),
            children: cs,
        })
    }

    // Both arguments are Explain AppliedIds.
    pub fn add_equation(&mut self, a: AppliedId, b: AppliedId, j: Justification) {
        let a_id = a.id;
        let b_id = b.id;

        let i = self.equations.len();
        let eq = Equation {
            l: a,
            r: b,
            j,
        };
        self.equations.push(eq);
    }

    // Subst contains Explain-AppliedIds.
    // This also returns an Explain-AppliedId.
    pub fn pattern_subst(&mut self, pat: &Pattern<L>, subst: &Subst) -> AppliedId {
        match &pat.node {
            ENodeOrPVar::ENode(n) => {
                let mut n = n.clone();
                let mut refs: Vec<&mut _> = n.applied_id_occurences_mut();
                assert_eq!(pat.children.len(), refs.len());
                for i in 0..refs.len() {
                    *(refs[i]) = self.pattern_subst(&pat.children[i], subst);
                }
                self.add_explain_enode(n)
            },
            ENodeOrPVar::PVar(v) => {
                subst[v].clone()
            },
        }
    }

    fn mk_identity_app_id(&self, i: Id) -> AppliedId {
        let slots = self.slots_of(i);
        let identity = SlotMap::identity(&slots);
        AppliedId::new(i, identity)
    }

    fn slots_of(&self, i: Id) -> HashSet<Slot> {
        self.term_id_to_enode[&i].slots()
    }

    fn incidence_map(&self) -> HashMap<Id, HashSet<EquationId>> {
        let mut out: HashMap<Id, HashSet<EquationId>> = HashMap::default();

        for (&i, _) in &self.term_id_to_enode {
            out.insert(i, HashSet::default());
        }

        for (i, Equation { l, r, .. }) in self.equations.iter().enumerate() {
            out.get_mut(&l.id).unwrap().insert(i);
            out.get_mut(&r.id).unwrap().insert(i);
        }

        out
    }

    fn find_explanation(&self, a: &AppliedId, b: &AppliedId, imap: &IMap) -> Explanation<L> {
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
