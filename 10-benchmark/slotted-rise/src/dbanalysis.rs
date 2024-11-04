use crate::*;

use std::collections::HashSet;

#[derive(PartialEq, Eq, Clone)]
pub struct DBRiseAnalysisData {
  pub free: HashSet<Index>,
  // TODO: add this for fair comparison, or remove from egg baseline?
  // beta_extract: RecExpr<DBRise>,
}

pub fn i32_from_eclass(egraph: &EGraph<DBRise, DBRiseAnalysisData>, id: Id) -> i32 {
  // TODO: use beta_extract result for fair comparison, or change egg baseline?
  for enode in egraph.enodes(id) {
    match enode {
      DBRise::Number(n) => return n,
      _ => ()
    }
  }
  panic!("expected Number in eclass")
}

pub fn eclass_get_var(egraph: &EGraph<DBRise, DBRiseAnalysisData>, id: Id) -> Option<Index> {
  for enode in egraph.enodes(id) {
    match enode {
      DBRise::Var(idx) => return Some(idx),
      _ => ()
    }
  }
  None
}

impl Analysis<DBRise> for DBRiseAnalysisData {
  fn make(eg: &EGraph<DBRise, Self>, enode: &DBRise) -> Self {
    let mut free = HashSet::default();

    match enode {
      DBRise::Var(v) => {
        free.insert(*v);
      }
      DBRise::Lam(a) => {
        free.extend(
          eg.analysis_data(a.id).free.iter().cloned()
            .filter(|&idx| idx != Index(0))
            .map(|idx| Index(idx.0 - 1)));
      }
      DBRise::Sigma(i, a, b) => {
        let i_num = i32_from_eclass(eg, i.id) as u32;
        let used = eg.analysis_data(a.id).free.contains(&Index(i_num));
        free.extend(
          eg.analysis_data(a.id).free.iter().cloned()
            .filter(|&idx| idx != Index(i_num))
            .map(|idx|
              if idx.0 > i_num {
                Index(idx.0 - 1)
              } else {
                idx
              }
            ));
        if used {
          free.extend(eg.analysis_data(b.id).free.iter()
            .map(|idx| {
              Index(idx.0 + i_num)
            }));
        }
      }
      DBRise::Phi(i, k, a) => {
        let i_num = i32_from_eclass(eg, i.id);
        let k_num = i32_from_eclass(eg, k.id);
        free.extend(
          eg.analysis_data(a.id).free.iter().cloned()
            .map(|idx| {
              let n = idx.0 as i32;
              if n >= k_num {
                Index((n + i_num) as u32)
              } else {
                idx
              }
            }));
      }
      _ => {
        for aid in enode.applied_id_occurences() {
          free.extend(&eg.analysis_data(aid.id).free);
        }
      }
    }

    DBRiseAnalysisData { free }
  }

  fn merge(mut l: Self, r: Self) -> Self {
    l.free.retain(|x| r.free.contains(x));
    l
  }
}