#![allow(unused)]

use crate::*;

// In the context of explanations, there is a bijection between Ids and Terms.
// Hence Ids uniquely identify certain concrete terms.

type EquationId = usize;

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

    // equations = (lhs, rhs, j). All rules are applied as lhs -> rhs.
    pub equations: Vec<(AppliedId, AppliedId, Justification)>,

    pub incidence_map: HashMap<Id, Vec<EquationId>>,
}

impl<L: Language> Default for Explain<L> {
    fn default() -> Self {
        Self {
            translator: Default::default(),
            enode_to_term_id: Default::default(),
            term_id_to_enode: Default::default(),
            equations: Default::default(),
            incidence_map: Default::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Justification {
    Congruence,
    Rule(String),
    Explicit, // union called without a rule.
}

impl<L: Language> EGraph<L> {
    pub fn explain_equivalence(&mut self, a: RecExpr<L>, b: RecExpr<L>) -> Explanation<L> {
        let a_ = self.add_expr(a.clone());
        let b_ = self.add_expr(b.clone());
        assert!(self.eq(&a_, &b_));
        assert!(self.explain.is_some());

        self.add_congruence_equations();

        let Some(explain) = self.explain.as_mut() else { panic!() };
        for (x, y, j) in &explain.equations {
            let x = explain.term_id_to_term(&x).unwrap();
            let y = explain.term_id_to_term(&y).unwrap();
            eprintln!("{} == {} by {:?}", x, y, j);
        }

        self.remove_congruence_equations();

        todo!()
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
            println!("add equation!");
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
        dbg!(&bt);

        bt
    }

    fn remove_congruence_equations(&mut self) {
        let Some(explain) = self.explain.as_mut() else { panic!() };
        explain.equations.retain(|(_, _, j)| !matches!(j, Justification::Congruence));
    }

    // get_justification_chain(a, b).last().unwrap().1 == b, whereas a doesn't come up in the list.
    // panics, if a and b are not equal.
    fn get_justification_chain(&self, a: AppliedId, b: AppliedId) -> Vec<(Justification, AppliedId)> {
        todo!()
    }
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
        self.equations.push((a, b, j));

        self.incidence_map.entry(a_id).or_default().push(i);
        self.incidence_map.entry(b_id).or_default().push(i);
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

}

#[derive(Debug)]
pub struct Explanation<L>(std::marker::PhantomData<L>);


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

