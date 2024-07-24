#![allow(unused)]

use crate::*;

// In the context of explanations, there is a bijection between Ids and Terms.
// Hence Ids uniquely identify certain concrete terms.

#[derive(Debug)]
pub struct Explain<L: Language> {
    // These two form a bijection:
    enode_to_term_id: HashMap<L/*shape*/, AppliedId>,
    term_id_to_enode: HashMap<Id, L/*with identity perm*/>,

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

    pub fn justify_union(&mut self, a: AppliedId, b: AppliedId, j: Justification) {
        todo!()
    }

    // get_justification_chain(a, b).last().unwrap().1 == b, whereas a doesn't come up in the list.
    // panics, if a and b are not equal.
    fn get_justification_chain(&self, a: AppliedId, b: AppliedId) -> Vec<(Justification, AppliedId)> {
        todo!()
    }
}
