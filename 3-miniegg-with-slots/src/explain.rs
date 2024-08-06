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
    translator: HashMap<Id, AppliedId>,

    // These two form a bijection:
    enode_to_term_id: HashMap<L/*shape*/, AppliedId>,
    term_id_to_enode: HashMap<Id, L/*with identity perm*/>,

    // equations = (lhs, rhs, j). All rules are applied as lhs -> rhs.
    equations: Vec<(AppliedId, AppliedId, Justification)>,

    incidence_map: HashMap<Id, Vec<EquationId>>,
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
    Explicit, // union called directly.
}

impl<L: Language> Explain<L> {
    pub fn translate(&self, l: &AppliedId) -> AppliedId {
        // l.m :: slots(l.id) -> X
        let a = &self.translator[&l.id];
        // a == l.id

        // a has some redundant slot choices that shouldn't be touched by l.m.
        let mut m = l.m.clone();
        for s in a.slots() {
            if !m.contains_key(s) {
                m.insert(s, s);
            }
        }
        a.apply_slotmap(&m)
    }

    pub fn translate_enode(&self, e: &L) -> L {
        e.map_applied_ids(|x| self.translate(&x))
    }

    pub fn add(&mut self, l: L) -> AppliedId {
        todo!()
    }

    pub fn add_translation(&mut self, l: L, i: AppliedId) -> AppliedId {
        todo!()
    }

    pub fn add_enode(&mut self, l: L, i: AppliedId) {
        { // enode_to_term_id
            let (sh, bij) = l.shape();
            //    l == i, and given l == sh * bij
            // -> sh * bij == i
            // -> sh == i * bij^-1
            self.enode_to_term_id.insert(sh, i.apply_slotmap(&bij.inverse()));
        }

        { // term_id_to_enode
            let l2 = l.apply_slotmap(&i.m.inverse());
            //    l == i, and given i == i.id * i.m
            // -> l == i.id * i.m
            // -> l * i.m^-1 == i.id
            self.term_id_to_enode.insert(i.id, l2);
        }

        self.incidence_map.insert(i.id, Vec::new());
    }

    pub fn enode_to_term_id(&self, l: &L) -> Option<AppliedId> {
        let (sh, bij) = l.shape();
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
        Some(x.apply_slotmap(&a.m))
    }

    pub fn add_equation(&mut self, a: AppliedId, b: AppliedId, j: Justification) {
        let a_id = a.id;
        let b_id = b.id;

        let i = self.equations.len();
        self.equations.push((a, b, j));

        self.incidence_map.get_mut(&a_id).unwrap().push(i);
        self.incidence_map.get_mut(&b_id).unwrap().push(i);
    }

    pub fn pattern_subst(&self, pat: &Pattern<L>, subst: &Subst) -> AppliedId {
        todo!()
    }

    pub fn explain_equivalence(&self, a: AppliedId, b: AppliedId) -> Option<Explanation<L>> {
        todo!()
    }

    // get_justification_chain(a, b).last().unwrap().1 == b, whereas a doesn't come up in the list.
    // panics, if a and b are not equal.
    fn get_justification_chain(&self, a: AppliedId, b: AppliedId) -> Vec<(Justification, AppliedId)> {
        todo!()
    }
}

pub struct Explanation<L>(std::marker::PhantomData<L>);
