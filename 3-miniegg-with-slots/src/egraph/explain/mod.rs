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

mod find;
pub use find::*;

// In the context of explanations, there is a bijection between Ids and Terms.
// Hence Ids uniquely identify certain concrete terms.

pub type EquationId = usize;
pub type IMap = HashMap<Id, HashSet<EquationId>>;

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
