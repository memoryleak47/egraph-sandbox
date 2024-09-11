#![allow(unused)]

use crate::*;

mod pre;
pub use pre::*;

mod explanation;
pub use explanation::*;

mod alpha;
pub use alpha::*;

mod record;
pub use record::*;

mod find_path;
pub use find_path::*;

mod interpret_path;
pub use interpret_path::*;

// In the context of explanations, there is a bijection between Ids and Terms.
// Hence Ids uniquely identify certain concrete terms.

pub type EquationId = usize;

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

    // incidence map. Is often empty.
    pub imap: HashMap<Id, HashSet<EquationId>>,
}

impl<L: Language> Default for Explain<L> {
    fn default() -> Self {
        Self {
            translator: Default::default(),
            enode_to_term_id: Default::default(),
            term_id_to_enode: Default::default(),
            equations: Default::default(),
            imap: Default::default(),
        }
    }
}

impl<L: Language> Explain<L> {
    pub fn find_explanation(&self, a: &AppliedId, b: &AppliedId) -> Explanation<L> {
        let p = self.find_path(a, b);
        self.interpret_path(p)
    }

    pub fn check_explain_app_id(&self, x: &AppliedId) {
        assert_eq!(self.slots_of(x.id), x.m.keys());
    }

    pub fn check_explain_enode(&self, x: &L) {
        for y in x.applied_id_occurences() {
            self.check_explain_app_id(&y);
        }
    }

    pub fn check_equation(&self, eq: &Equation) {
        self.check_explain_app_id(&eq.l);
        self.check_explain_app_id(&eq.r);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Justification {
    Congruence,
    Rule(String),
    Explicit, // union called without a rule.
}
