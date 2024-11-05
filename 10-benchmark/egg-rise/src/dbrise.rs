use egg::*;
use std::collections::HashSet;
use std::cmp::Ordering;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Copy)]
pub struct Index(pub u32);

impl std::str::FromStr for Index {
    type Err = Option<std::num::ParseIntError>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("%") {
            s["%".len()..].parse().map(Index).map_err(Some)
        } else {
            Err(None)
        }
    }
}

impl std::fmt::Display for Index {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "%{}", self.0)
    }
}

// De-Bruijn Rise
define_language! {
    pub enum DBRise {
        Var(Index),
        "app" = App([Id; 2]),
        "lam" = Lambda(Id),

        "sig" = Sigma([Id; 3]),
        "phi" = Phi([Id; 3]),

        Number(i32),
        Symbol(Symbol),
    }
}

pub type DBRiseEGraph = EGraph<DBRise, DBRiseAnalysis>;
pub type DBRiseExpr = RecExpr<DBRise>;

#[derive(Default)]
pub struct DBRiseAnalysis;

#[derive(Default, Debug)]
pub struct DBData {
    pub free: HashSet<Index>,
    pub beta_extract: DBRiseExpr,
}

pub fn i32_from_eclass(egraph: &DBRiseEGraph, id: Id) -> i32 {
    match egraph[id].data.beta_extract.as_ref() {
        &[DBRise::Number(i_num)] => i_num,
        _ => panic!("expected Number in eclass")
    }
}

impl Analysis<DBRise> for DBRiseAnalysis {
    type Data = DBData;

    fn merge(&mut self, to: &mut DBData, from: DBData) -> DidMerge {
        let before_len = to.free.len();
        to.free.retain(|x| from.free.contains(x));
        // overly conservative: to.free.extend(from.free);
        let mut did_change = before_len != to.free.len();
        if !from.beta_extract.as_ref().is_empty() &&
            (to.beta_extract.as_ref().is_empty() ||
                to.beta_extract.as_ref().len() > from.beta_extract.as_ref().len()) {
            to.beta_extract = from.beta_extract;
            did_change = true;
        }
        DidMerge(did_change, true) // TODO: more precise second bool
    }

    fn make(egraph: &DBRiseEGraph, enode: &DBRise) -> DBData {
        let mut free = HashSet::default();
        match enode {
            DBRise::Var(v) => {
                free.insert(*v);
            }
            DBRise::Lambda(a) => {
                free.extend(
                    egraph[*a].data.free.iter().cloned()
                        .filter(|&idx| idx != Index(0))
                        .map(|idx| Index(idx.0 - 1)));
            }
            DBRise::Sigma([i, a, b]) => {
                let i_num = i32_from_eclass(egraph, *i) as u32;
                let used = egraph[*a].data.free.contains(&Index(i_num));
                free.extend(
                    egraph[*a].data.free.iter().cloned()
                        .filter(|&idx| idx != Index(i_num))
                        .map(|idx| if idx.0 > i_num { Index(idx.0 - 1) } else { idx }));
                if used {
                    free.extend(egraph[*b].data.free.iter().map(|idx| {
                        Index(idx.0 + i_num)
                    }));
                }
            }
            DBRise::Phi([i, k, a]) => {
                let i_num = i32_from_eclass(egraph, *i);
                let k_num = i32_from_eclass(egraph, *k);
                free.extend(
                    egraph[*a].data.free.iter().cloned().map(|idx| {
                        let n = idx.0 as i32;
                        if n >= k_num { Index((n + i_num) as u32) } else { idx }
                    }));
            }
            _ => {
                enode.for_each(|c| free.extend(&egraph[c].data.free));
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
        DBData { free, beta_extract }
    }
}

pub fn add(to: &mut Vec<DBRise>, e: DBRise) -> Id {
    to.push(e);
    Id::from(to.len() - 1)
}

pub fn add_expr(to: &mut Vec<DBRise>, e: &[DBRise]) -> Id {
    let offset = to.len();
    to.extend(e.iter().map(|n| n.clone().map_children(|id| {
        Id::from(usize::from(id) + offset)
    })));
    Id::from(to.len() - 1)
}
