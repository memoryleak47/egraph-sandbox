use crate::*;

use std::collections::HashSet;

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Constant {
  Var(Index),
  Number(i32),
}

#[derive(PartialEq, Eq, Clone)]
pub struct DBRiseAnalysisData {
  pub free: HashSet<Index>,
  // TODO: use this instead of constant for fair comparison, or change egg baseline?
  // beta_extract: RecExpr<DBRise>,
  pub constant: Option<Constant>,
}

pub fn i32_from_eclass(egraph: &EGraph<DBRise, DBRiseAnalysisData>, id: Id) -> i32 {
  /*
  Method 1: iterate through enodes in O(n)

  for enode in egraph.enodes(id) {
    match enode {
      DBRise::Number(n) => return n,
      _ => ()
    }
  }
  */
  // Method 2: use analysis result in O(1)
  match egraph.analysis_data(id).constant {
    Some(Constant::Number(n)) => return n,
    _ => ()
  }
  panic!("expected Number in eclass")
}

pub fn eclass_get_var(egraph: &EGraph<DBRise, DBRiseAnalysisData>, id: Id) -> Option<Index> {
  /* Method 1
  for enode in egraph.enodes(id) {
    match enode {
      DBRise::Var(idx) => return Some(idx),
      _ => ()
    }
  }
  */
  // Method 2
  match egraph.analysis_data(id).constant {
    Some(Constant::Var(idx)) => return Some(idx),
    _ => ()
  }
  None
}

impl Analysis<DBRise> for DBRiseAnalysisData {
  fn make(eg: &EGraph<DBRise, Self>, enode: &DBRise) -> Self {
    let mut free = HashSet::default();
    let mut constant = None;

    match enode {
      DBRise::Var(v) => {
        free.insert(*v);
        constant = Some(Constant::Var(*v));
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
      DBRise::Number(n) => {
        constant = Some(Constant::Number(*n));
      }
      DBRise::App(_, _) | DBRise::Symbol(_) => {
        for aid in enode.applied_id_occurences() {
          free.extend(&eg.analysis_data(aid.id).free);
        }
      }
    }

    DBRiseAnalysisData { free, constant }
  }

  fn merge(mut l: Self, r: Self) -> Self {
    l.free.retain(|x| r.free.contains(x));
    match (&l.constant, &r.constant) {
      (None, None) => {}
      (Some(_), None) => {}
      (None, Some(_)) => { l.constant = r.constant }
      (Some(a), Some(b)) => { assert_eq!(a, b) }
    };
    l
  }
}