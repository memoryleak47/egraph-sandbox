use crate::*;

type DBRewrite = Rewrite<DBRise, DBRiseAnalysisData>;

/*
pub enum RiseSubstMethod {
    Extraction,
    SmallStep, // <<< only this implemented
    SmallStepUnoptimized,
}
*/

pub fn db_rise_rules() -> Vec<DBRewrite> {
  let mut rewrites = Vec::new();

  rewrites.push(eta());
  // rewrites.push(eta_expansion());

  rewrites.push(map_fusion());
  rewrites.push(map_fission());

  rewrites.push(beta());

  rewrites.push(sig_unused());
  rewrites.push(phi_unused());
  rewrites.push(sig_lam());
  rewrites.push(phi_lam());
  rewrites.push(sig_app());
  rewrites.push(phi_app());
  rewrites.push(sig_var());
  rewrites.push(phi_var());

  rewrites
}

fn not_free_in(eg: &EGraph<DBRise, DBRiseAnalysisData>, i: Index, b: &AppliedId) -> bool {
  !eg.analysis_data(b.id).free.contains(&i)
}

fn range_free_in(eg: &EGraph<DBRise, DBRiseAnalysisData>, i: &AppliedId, b: &AppliedId) -> bool {
  let idx = i32_from_eclass(eg, i.id) as u32;
  let bmax = eg.analysis_data(b.id).free.iter().max();
  if let Some(bmax) = bmax {
    bmax.0 >= idx
  } else {
    false
  }
}

fn beta() -> DBRewrite {
  Rewrite::new("beta",
    "(app (lam ?body) ?e)",
    "(sig 0 ?body ?e)")
}

fn eta() -> DBRewrite {
  Rewrite::new_if("eta",
    "(lam (app ?f %0))",
    "(phi -1 1 ?f)",
    |subst, eg| {
      not_free_in(eg, Index(0), &subst["f"])
    })
}

fn eta_expansion() -> DBRewrite {
  Rewrite::new("eta-expansion",
    "?f",
    "(lam (app ?f %0))")
}

fn sig_unused() -> DBRewrite {
  Rewrite::new_if("sig-unused",
    "(sig ?i ?body ?e)",
    "?body",
    |subst, eg| {
      !range_free_in(eg, &subst["i"], &subst["body"])
    })
}

fn phi_unused() -> DBRewrite {
  Rewrite::new_if("phi-unused",
    "(phi ?i ?j ?body)",
    "?body",
    |subst, eg| {
      !range_free_in(eg, &subst["j"], &subst["body"])
    })
}

fn filter_map_rewrite<L: Language + 'static, N: Analysis<L> + 'static>(
  rule: &str, a: &str, b: &str,
  f: impl Fn(Subst, &mut EGraph<L, N>) -> Option<Subst> + 'static
) -> Rewrite<L, N> {
  let a = Pattern::parse(a).unwrap();
  let b = Pattern::parse(b).unwrap();
  let rule = rule.to_string();
  let a2 = a.clone();
  RewriteT {
    searcher: Box::new(move |eg| ematch_all(eg, &a)),
    applier: Box::new(move |substs, eg| {
      for subst in substs {
        if let Some(subst2) = f(subst, eg) {
          eg.union_instantiations(&a2, &b, &subst2, Some(rule.to_string()));
        }
      }
    }),
  }.into()
}

fn subst_shift_number(
  eg: &mut EGraph<DBRise, DBRiseAnalysisData>, mut subst: Subst,
  var: &str, shift: i32, new_var: &str
) -> Subst {
  let shifted = DBRise::Number(i32_from_eclass(eg, subst[var].id) + shift);
  subst.insert(new_var.to_owned(), eg.add(shifted));
  subst
}

fn sig_lam() -> DBRewrite {
  filter_map_rewrite("sig-lam",
    "(sig ?i (lam ?a) ?b)",
    "(lam (sig ?ip1 ?a ?b))",
    // with ?ip1:
    |subst, eg| {
      if range_free_in(eg, &subst["i"], &subst["a"]) {
        Some(subst_shift_number(eg, subst, "i", 1, "ip1"))
      } else {
        None
      }
    })
}

fn phi_lam() -> DBRewrite {
  filter_map_rewrite("phi-lam",
    "(phi ?i ?k (lam ?a))",
    "(lam (phi ?i ?kp1 ?a))",
    // with ?kp1:
    |subst, eg| {
      if range_free_in(eg, &subst["k"], &subst["a"]) {
        Some(subst_shift_number(eg, subst, "k", 1, "kp1"))
      } else {
        None
      }
    })
}

fn sig_app() -> DBRewrite {
    Rewrite::new_if("sig-app",
      "(sig ?i (app ?a1 ?a2) ?b)",
      "(app (sig ?i ?a1 ?b) (sig ?i ?a2 ?b))",
      |subst, eg| {
        range_free_in(eg, &subst["i"], &subst["a1"]) ||
        range_free_in(eg, &subst["i"], &subst["a2"])
      })
}

fn phi_app() -> DBRewrite {
  Rewrite::new_if("phi-app",
    "(phi ?i ?k (app ?a ?b))",
    "(app (phi ?i ?k ?a) (phi ?i ?k ?b))",
    |subst, eg| {
      range_free_in(eg, &subst["k"], &subst["a"]) ||
      range_free_in(eg, &subst["k"], &subst["b"])
    })
}

// TODO? pattern matching phase could be shared with sig_unused
fn sig_var() -> DBRewrite {
  filter_map_rewrite("sig-var",
    "(sig ?i ?n ?b)",
    "?res",
    |mut subst, eg| {
      if range_free_in(eg, &subst["i"], &subst["n"]) {
        eclass_get_var(eg, subst["n"].id).map(move |Index(idx)| {
          let i_num = i32_from_eclass(eg, subst["i"].id);
          let n = idx as i32;
          let node = if n > i_num {
            DBRise::Var(Index(idx - 1))
          } else if n == i_num {
            DBRise::Phi(subst["i"].clone(), eg.add(DBRise::Number(0)), subst["b"].clone())
          } else {
            // DBRise::Var(Index(var))
            unreachable!()
          };
          subst.insert("res".to_owned(), eg.add(node));
          subst
        })
      } else {
        None
      }
    }
  )
}

// TODO? pattern matching phase could be shared with phi_unused
fn phi_var() -> DBRewrite {
  filter_map_rewrite("sig-var",
    "(phi ?i ?k ?n)",
    "?res",
    |mut subst, eg| {
      if range_free_in(eg, &subst["k"], &subst["n"]) {
        eclass_get_var(eg, subst["n"].id).map(|Index(idx)| {
          let i_num = i32_from_eclass(eg, subst["i"].id);
          let k_num = i32_from_eclass(eg, subst["k"].id);
          let n = idx as i32;
          let shifted = DBRise::Var(Index(if n >= k_num { (n + i_num) as u32 } else { unreachable!() }));
          subst.insert("res".to_owned(), eg.add(shifted));
          subst
        })
      } else {
        None
      }
    }
  )
}

/////////////////////

fn map_fusion() -> DBRewrite {
    Rewrite::new("map-fusion",
      "(app (app map ?f) (app (app map ?g) ?arg))",
      "(app (app map (lam (app (phi 1 0 ?f) (app (phi 1 0 ?g) %0)))) ?arg)")
}

fn map_fission() -> DBRewrite {
    Rewrite::new_if("map-fission",
      "(app map (lam (app ?f ?gx)))",
      "(lam (app (app map ?f) (app (app map (lam (phi 1 1 ?gx))) %0)))",
      |subst, eg| {
        not_free_in(eg, Index(0), &subst["f"])
      })
}
