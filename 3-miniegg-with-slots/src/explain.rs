#![allow(unused)]

use crate::*;

// In the context of explanations, there is a bijection between Ids and Terms.
// Hence Ids uniquely identify certain concrete terms.

#[derive(Debug)]
pub struct Explain<L: Language> {
    // These two form a bijection:
    enode_to_term_id: HashMap<L, AppliedId>,
    term_id_to_enode: HashMap<AppliedId, L>,

    // justification_forest[x][y] returns the justification for unifying x and y.
    // justification_forest[x][y] is stored redundantly with justification_forest[y][x].
    justification_forest: HashMap<Id, HashMap<AppliedId, Justification>>,

    // For each permutation, remembers how we computed it.
    perm_justifications: HashMap<Id, HashMap<Perm, PermJustification>>,
}

impl<L: Language> Default for Explain<L> {
    fn default() -> Self {
        Self {
            enode_to_term_id: Default::default(),
            term_id_to_enode: Default::default(),
            justification_forest: Default::default(),
            perm_justifications: Default::default(),
        }
    }
}

#[derive(Debug)]
pub enum Justification {
    Congruence,
    Rule(String, /*forward / backward*/ bool),
}

#[derive(Debug)]
pub enum PermJustification {
    Rule(String, /*forward / backward*/ bool),

    Composition(Perm, Perm),
    Inverse(Perm),

    // the perm was generated for another class, which was then unioned with this one.
    Equality(Id),
}

impl<L: Language> Explain<L> {
    // TODO add stuff to the bijection.
    pub fn add_enode(&mut self, l: L, i: AppliedId) {
        todo!()
    }

    // TODO normalize slots somehow
    pub fn enode_to_term_id(&self, _: &L) -> Option<AppliedId> {
        todo!()
    }

    // TODO normalize slots somehow
    pub fn term_id_to_enode(&self, _: &AppliedId) -> Option<L> {
        todo!()
    }

    pub fn justify_union(&mut self, a: AppliedId, b: AppliedId, j: Justification) {
        todo!()
    }

    // get_justification_chain(a, b).last().unwrap().1 == b, whereas a doesn't come up in the list.
    // panics, if a and b are not equal.
    fn get_justification_chain(&self, a: AppliedId, b: AppliedId) -> Vec<(Justification, AppliedId)> {
        todo!()
    }
}
