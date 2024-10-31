use crate::*;

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum DBRise {
    // lambda calculus:
    Lam(AppliedId),
    App(AppliedId, AppliedId),
    Var(Index),
    Sigma(Index, AppliedId, AppliedId),
    Phi(Index, Index, AppliedId),

    // rest:
    Number(u32),
    Symbol(Symbol),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Copy)]
pub struct Index(pub u32);

impl std::fmt::Display for Index {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "%{}", self.0)
    }
}

impl std::str::FromStr for Index {
    type Err = Option<std::num::ParseIntError>;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if s.starts_with("%") {
            s["%".len()..].parse().map(Index).map_err(Some)
        } else {
            Err(None)
        }
    }
}

impl Language for DBRise {
    fn all_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        let mut out = Vec::new();
        match self {
            DBRise::Lam(b) => {
                out.extend(b.slots_mut());
            }
            DBRise::App(l, r) => {
                out.extend(l.slots_mut());
                out.extend(r.slots_mut());
            }
            DBRise::Var(_) => {}
            DBRise::Sigma(_, a, b) => {
                out.extend(a.slots_mut());
                out.extend(b.slots_mut());
            }
            DBRise::Phi(_, _, a) => {
                out.extend(a.slots_mut());
            }
            DBRise::Number(_) => {}
            DBRise::Symbol(_) => {}
        }
        out
    }

    fn public_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        self.all_slot_occurences_mut()
    }

    fn applied_id_occurences_mut(&mut self) -> Vec<&mut AppliedId> {
        match self {
            DBRise::Lam(b) => vec![b],
            DBRise::App(l, r) => vec![l, r],
            DBRise::Var(_) => vec![],
            DBRise::Sigma(_, a, b) => vec![a, b],
            DBRise::Phi(_, _, a) => vec![a],
            DBRise::Number(_) => vec![],
            DBRise::Symbol(_) => vec![],
        }
    }

    fn to_op(&self) -> (String, Vec<Child>) {
        match self.clone() {
            DBRise::Lam(b) => (String::from("lam"), vec![Child::AppliedId(b)]),
            DBRise::App(l, r) => (String::from("app"), vec![Child::AppliedId(l), Child::AppliedId(r)]),
            DBRise::Var(s) => (format!("{s}"), vec![]),
            DBRise::Sigma(i, a, b) => (format!("sig-{}", i), vec![Child::AppliedId(a), Child::AppliedId(b)]),
            // TODO: (String::from("phi"), vec![Child::Custom(i), Child::Cusom(k), Child::AppliedId(a)]) ?
            DBRise::Phi(i, k, a) => (format!("phi-{}-{}", i, k), vec![Child::AppliedId(a)]),
            DBRise::Number(n) => (format!("{}", n), vec![]),
            DBRise::Symbol(s) => (format!("{}", s), vec![]),
        }
    }

    fn from_op(op: &str, children: Vec<Child>) -> Option<Self> {
        unimplemented!()
        /*
        match (op, &*children) {
            ("lam", [Child::Slot(s), Child::AppliedId(a)]) => Some(Rise::Lam(*s, a.clone())),
            ("app", [Child::AppliedId(l), Child::AppliedId(r)]) => Some(Rise::App(l.clone(), r.clone())),
            ("var", [Child::Slot(s)]) => Some(Rise::Var(*s)),
            ("let", [Child::Slot(s), Child::AppliedId(t), Child::AppliedId(b)]) => Some(Rise::Let(*s, t.clone(), b.clone())),
            (op, []) => {
                if let Ok(u) = op.parse::<u32>() {
                    Some(Rise::Number(u))
                } else {
                    let s: Symbol = op.parse().ok()?;
                    Some(Rise::Symbol(s))
                }
            },
            _ => None,
        }
            */
    }
}

use std::fmt::*;

impl Debug for DBRise {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            DBRise::Lam(b) => write!(f, "(lam {b:?})"),
            DBRise::App(l, r) => write!(f, "(app {l:?} {r:?})"),
            DBRise::Var(s) => write!(f, "{s}"),
            DBRise::Sigma(i, a, b) => write!(f, "(sig {i} {a:?} {b:?})"),
            DBRise::Phi(i, k, a) => write!(f, "(phi {i} {k} {a:?})"),
            DBRise::Number(i) => write!(f, "{i}"),
            DBRise::Symbol(i) => write!(f, "symb{i:?}"),
        }
    }
}


