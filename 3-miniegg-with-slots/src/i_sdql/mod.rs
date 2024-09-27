#![allow(unused)]
#![allow(non_snake_case)]

use crate::*;

mod rewrite;
pub use rewrite::*;

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum SdqlEnode {
    Lam(Slot, AppliedId),
    Var(Slot),
    Sing(AppliedId, AppliedId),
    Sum(Slot, Slot, /*range: */AppliedId, /*body: */ AppliedId),
}

impl Language for SdqlEnode {
    fn all_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        let mut out = Vec::new();
        match self {
            SdqlEnode::Lam(x, b) => {
                out.push(x);
                out.extend(b.slots_mut());
            }
            SdqlEnode::Var(x) => {
                out.push(x);
            }
            SdqlEnode::Sing(x, y) => {
                out.extend(x.slots_mut());
                out.extend(y.slots_mut());
            }
            SdqlEnode::Sum(k, v, r, b) => {
                out.push(k);
                out.push(v);
                out.extend(r.slots_mut());
                out.extend(b.slots_mut());
            }
        }
        out
    }

    fn public_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        let mut out = Vec::new();
        match self {
            SdqlEnode::Lam(x, b) => {
                out.extend(b.slots_mut().into_iter().filter(|y| *y != x));

            }
            SdqlEnode::Var(x) => {
                out.push(x);
            }
            SdqlEnode::Sing(x, y) => {
                out.extend(x.slots_mut());
                out.extend(y.slots_mut());
            }
            SdqlEnode::Sum(k, v, r, b) => {
                out.extend(b.slots_mut().into_iter().filter(|y| *y != k && *y != v));
                out.extend(r.slots_mut());
            }
        }
        out
    }

    fn applied_id_occurences_mut(&mut self) -> Vec<&mut AppliedId> {
        match self {
            SdqlEnode::Lam(_, y) => vec![y],
            SdqlEnode::Var(_) => vec![],
            SdqlEnode::Sing(x, y) => vec![x, y],
            SdqlEnode::Sum(_, _, r, b) => vec![r, b],
        }
    }

    fn to_op(&self) -> (String, Vec<Child>) {
        match self.clone() {
            SdqlEnode::Lam(s, a) => (String::from("lambda"), vec![Child::Slot(s), Child::AppliedId(a)]),
            SdqlEnode::Var(s) => (String::from("var"), vec![Child::Slot(s)]),
            SdqlEnode::Sing(x, y) => (String::from("sing"), vec![Child::AppliedId(x), Child::AppliedId(y)]),
            SdqlEnode::Sum(k, v, r, b) => (String::from("sum"), vec![Child::Slot(k), Child::Slot(v), Child::AppliedId(r), Child::AppliedId(b)]),
        }
    }

    fn from_op(op: &str, children: Vec<Child>) -> Option<Self> {
        match (op, &*children) {
            ("lambda", [Child::Slot(s), Child::AppliedId(a)]) => Some(SdqlEnode::Lam(*s, a.clone())),
            ("var", [Child::Slot(s)]) => Some(SdqlEnode::Var(*s)),
            ("sing", [Child::AppliedId(x), Child::AppliedId(y)]) => Some(SdqlEnode::Sing(x.clone(), y.clone())),
            ("sum", [Child::Slot(k), Child::Slot(v), Child::AppliedId(r), Child::AppliedId(b)]) => Some(SdqlEnode::Sum(*k, *v, r.clone(), b.clone())),
            _ => None,
        }
    }
}
