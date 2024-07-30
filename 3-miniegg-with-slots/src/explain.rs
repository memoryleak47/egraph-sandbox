#![allow(unused)]

use crate::*;

// In the context of explanations, there is a bijection between Ids and Terms.
// Hence Ids uniquely identify certain concrete terms.

type EquationId = usize;

#[derive(Debug)]
pub struct Explain<L: Language> {
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

    // get_justification_chain(a, b).last().unwrap().1 == b, whereas a doesn't come up in the list.
    // panics, if a and b are not equal.
    fn get_justification_chain(&self, a: AppliedId, b: AppliedId) -> Vec<(Justification, AppliedId)> {
        todo!()
    }
}
