use egg::*;
use std::collections::HashSet;
use std::cmp::Ordering;

define_language! {
    pub enum Rise {
        "var" = Var(Id),
        "app" = App([Id; 2]),
        "lam" = Lambda([Id; 2]),

        "let" = Let([Id; 3]),
        // "fix"

        ">>" = Then([Id; 2]),

        Number(i32),
        Symbol(Symbol),
    }
}

pub type RiseEGraph = EGraph<Rise, RiseAnalysis>;

#[derive(Default)]
pub struct RiseAnalysis;

#[derive(Default, Debug)]
pub struct Data {
    pub free: HashSet<Id>,
    pub beta_extract: RecExpr<Rise>,
}

impl Analysis<Rise> for RiseAnalysis {
    type Data = Data;

    fn merge(&mut self, to: &mut Data, from: Data) -> DidMerge {
        let before_len = to.free.len();
        to.free.retain(|x| from.free.contains(x));
        let mut did_change = before_len != to.free.len();
        if !from.beta_extract.as_ref().is_empty() &&
            (to.beta_extract.as_ref().is_empty() ||
                to.beta_extract.as_ref().len() > from.beta_extract.as_ref().len()) {
            to.beta_extract = from.beta_extract;
            did_change = true;
        }
        DidMerge(did_change, true) // TODO: more precise second bool
    }

    fn make(egraph: &RiseEGraph, enode: &Rise) -> Data {
        let extend = |free: &mut HashSet<Id>, i: &Id| {
            free.extend(&egraph[*i].data.free);
        };
        let mut free = HashSet::default();
        match enode {
            Rise::Var(v) => {
                free.insert(*v);
            }
            Rise::Lambda([v, a]) => {
                extend(&mut free, a);
                free.remove(v);
            }
            Rise::Let([v, a, b]) => {
                extend(&mut free, b);
                if free.remove(v) {
                    extend(&mut free, a);
                }
            }
            _ => {
                enode.for_each(|c| extend(&mut free, &c));
            }
        }
        let empty = enode.any(|id| {
            egraph[id].data.beta_extract.as_ref().is_empty()
        });
        let beta_extract = if empty {
            vec![].into()
        } else {
            enode.join_recexprs(|id| egraph[id].data.beta_extract.as_ref())
        };
        Data { free, beta_extract }
    }
}

pub fn unwrap_symbol(n: &Rise) -> Symbol {
    match n {
        &Rise::Symbol(s) => s,
        _ => panic!("expected symbol")
    }
}
