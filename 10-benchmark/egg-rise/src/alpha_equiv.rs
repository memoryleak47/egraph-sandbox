use egg::*;
use crate::rise::*;
use std::collections::{HashMap, HashSet};

static mut COUNTER: u32 = 0;
pub fn fresh_id() -> u32 {
    unsafe {
        let c = COUNTER;
        COUNTER += 1;
        c
    }
}

pub fn expr_to_alpha_equiv_pattern(e: RecExpr<Rise>) -> Pattern<Rise> {
    use std::str::FromStr;

    // println!("{}", e);
    let p = Pattern::<Rise>::from(
        expr_alpha_rename(e, ENodeOrVar::ENode,
                          |s| {
                              let s2 = format!("?{}", s.as_str());
                              ENodeOrVar::Var(Var::from_str(s2.as_str()).unwrap())
                          },
                          |s| {
                              ENodeOrVar::ENode(Rise::Symbol(s))
                          }));
    println!("{}", p);
    p
}

pub fn expr_fresh_alpha_rename(e: RecExpr<Rise>) -> RecExpr<Rise> {
    expr_alpha_rename(e, |r| r,
                      |_s| {
                          let s2 = format!("x{}", fresh_id());
                          Rise::Symbol(s2.into())
                      },
                      |s| Rise::Symbol(s))
}

pub fn expr_alpha_rename<L, BS, FS>(
    e: RecExpr<Rise>,
    init: impl Fn(Rise) -> L,
    bound_symbol: BS,
    free_symbol: FS) -> RecExpr<L>
    where L: Language, BS: Fn(Symbol) -> L, FS: Fn(Symbol) -> L
{
    let original_vec = e.as_ref();
    let mut new_vec = Vec::new();
    new_vec.extend(original_vec.iter().cloned().map(init));

    struct Env<'a, L, BS, FS> {
        original_vec: &'a [Rise],
        new_vec: &'a mut [L],
        sym_map: &'a mut HashMap<Symbol, L>,
        bound_symbol: BS,
        free_symbol: FS,
    }
    fn rec<L, BS, FS>(index: usize, env: &mut Env<L, BS, FS>)
        where L: Language, BS: Fn(Symbol) -> L, FS: Fn(Symbol) -> L
    {
        match env.original_vec[index] {
            Rise::Var(id) => rec(id.into(), env),
            Rise::App([f, e]) => {
                rec(f.into(), env);
                rec(e.into(), env);
            }
            Rise::Then([f, g]) => {
                rec(f.into(), env);
                rec(g.into(), env);
            }
            Rise::Lambda([x, e]) => {
                let s = match env.original_vec[usize::from(x)] {
                    Rise::Symbol(s) => s,
                    _ => panic!("expected symbol for lambda")
                };
                if env.sym_map.insert(s, (env.bound_symbol)(s)).is_some() {
                    panic!("symbol duplicate");
                }
                rec(x.into(), env);
                rec(e.into(), env);
            }
            Rise::Let(_) => { // [x, e, b]
                unimplemented!();
                /* let s = match env.original_vec[usize::from(x)] {
                    Rise::Symbol(s) => s,
                    _ => panic!("expected symbol for let")
                };
                if env.sym_map.insert(s, (env.bound_symbol)(s)).is_some() {
                    panic!("symbol duplicate");
                }
                rec(x.into(), env);
                rec(e.into(), env);
                rec(b.into(), env); */
            }
            Rise::Symbol(sym) => {
                env.new_vec[index] = env.sym_map.get(&sym).cloned().unwrap_or_else(|| (env.free_symbol)(sym));
            }
            Rise::Number(_) => ()
        }
    }

    rec(original_vec.len() - 1, &mut Env {
        original_vec,
        new_vec: &mut new_vec[..],
        sym_map: &mut HashMap::new(),
        bound_symbol,
        free_symbol
    });
    new_vec.into()
}

pub fn count_alpha_equiv(egraph: &mut RiseEGraph) -> usize {
    type Constraints = Vec<Constraint>;
    #[derive(Clone, PartialEq, Debug)]
    enum Constraint {
        Same(Id, Id),
        All(Vec<Constraint>),
        Any(Vec<Constraint>),
        Reverse(Box<Constraint>), // Same(x, y) becomes Same(y, x)
    }

    fn satisfied() -> Constraint {
        Constraint::All(vec![])
    }

    fn unsatisfied() -> Constraint {
        Constraint::Any(vec![])
    }

    fn must_be(cond: bool) -> Constraint {
        if cond { satisfied() } else { unsatisfied() }
    }

    fn constraint_under_bindings(c: &Constraint, a: Id, b: Id) -> Constraint {
        match c {
            &Constraint::Same(x, y) if x == a || y == b =>
                must_be(x == a && y == b),
            &Constraint::Same(x, y) =>
                Constraint::Same(x, y),
            Constraint::All(cs) =>
                Constraint::All(cs.iter().map(|c| constraint_under_bindings(c, a, b)).collect()),
            Constraint::Any(cs) =>
                Constraint::Any(cs.iter().map(|c| constraint_under_bindings(c, a, b)).collect()),
            Constraint::Reverse(c) =>
                Constraint::Reverse(Box::new(constraint_under_bindings(c, b, a)))
        }
    }

    // Same(x, x) is satisfied if x is a local free variable
    fn locally_satisfied(c: &Constraint) -> bool {
        fn constraint_if_open(c: &Constraint) -> Constraint {
            match c {
                &Constraint::Same(x, y) => must_be(x == y),
                Constraint::All(cs) =>
                    Constraint::All(cs.iter().map(constraint_if_open).collect()),
                Constraint::Any(cs) =>
                    Constraint::Any(cs.iter().map(constraint_if_open).collect()),
                Constraint::Reverse(c) =>
                    Constraint::Reverse(Box::new(constraint_if_open(c)))
            }
        }
        let c2 = partially_evaluate_constraint(constraint_if_open(c));
        c2 == satisfied()
    }

    fn partially_evaluate_constraint(c: Constraint) -> Constraint {
        match c {
            Constraint::Same(x, y) => Constraint::Same(x, y),
            Constraint::All(cs) => {
                let cs2: Vec<_> = cs.into_iter().map(partially_evaluate_constraint).collect();
                if cs2.iter().any(|c| c == &unsatisfied()) { return unsatisfied(); }
                if cs2.iter().all(|c| c == &satisfied()) { return satisfied(); }
                Constraint::All(cs2)
            }
            Constraint::Any(cs) => {
                let cs2: Vec<_> = cs.into_iter().map(partially_evaluate_constraint).collect();
                if cs2.iter().all(|c| c == &unsatisfied()) { return unsatisfied(); }
                if cs2.iter().any(|c| c == &satisfied()) { return satisfied(); }
                Constraint::Any(cs2)
            }
            Constraint::Reverse(c) => {
                let c2 = partially_evaluate_constraint(c.as_ref().clone());
                if c2 == unsatisfied() { return unsatisfied(); }
                if c2 == satisfied() { return satisfied(); }
                Constraint::Reverse(Box::new(c2))
            }
        }
    }

    fn eclasses(a: Id, b: Id,
                map: &mut HashMap<(Id, Id), Constraint>,
                equivs: &mut HashSet<Rise>,
                egraph: &RiseEGraph) -> Constraint {
        let a = egraph.find(a);
        let b = egraph.find(b);
        //if a == b { return satisfied(); }
        let swap = usize::from(a) < usize::from(b);
        let (a, b) = if swap { (b, a) } else { (a, b) };
        let c = match map.get(&(a, b)) {
            Some(c) => c.clone(),
            None => {
                map.insert((a, b), satisfied()); // cycles add no constraint
                let ans = egraph[a].nodes.iter().cloned();
                let bns = egraph[b].nodes.iter().cloned();
                let c = Constraint::Any(ans.zip(bns)
                    .map(|(an, bn)| {
                        let c = enodes(an.clone(), bn.clone(), map, equivs, egraph);
                        let c = partially_evaluate_constraint(c);
                        let same_but_not_same_id = locally_satisfied(&c) && a != b;
                        if same_but_not_same_id {
                            equivs.insert(an); equivs.insert(bn);
                        }
                        c
                    })
                    .collect());
                let c = partially_evaluate_constraint(c);
                map.insert((a, b), c.clone());
                c
            }
        };
        if swap { Constraint::Reverse(Box::new(c)) } else { c }
    }

    fn enodes(a: Rise, b: Rise,
              map: &mut HashMap<(Id, Id), Constraint>,
              equivs: &mut HashSet<Rise>,
              egraph: &RiseEGraph) -> Constraint {
        match (a, b) {
            // TODO: free variables
            (Rise::Var(x1), Rise::Var(x2)) => Constraint::Same(x1, x2),
            (Rise::App([f1, e1]), Rise::App([f2, e2])) => {
                let fc = eclasses(f1, f2, map, equivs, egraph);
                let ec = eclasses(e1, e2, map, equivs, egraph);
                Constraint::All(vec![fc, ec])
            }
            (Rise::Lambda([x1, e1]), Rise::Lambda([x2, e2])) => {
                let ec = eclasses(e1, e2, map, equivs, egraph);
                let c = constraint_under_bindings(&ec, x1, x2);
                // println!("lam {:?} -> {:?}", ec, c);
                c
            }
            (Rise::Symbol(s1), Rise::Symbol(s2)) => must_be(s1 == s2),
            (Rise::Number(n1), Rise::Number(n2)) => must_be(n1 == n2),
            (Rise::Then([f1, g1]), Rise::Then([f2, g2])) => {
                let fc = eclasses(f1, f2, map, equivs, egraph);
                let gc = eclasses(g1, g2, map, equivs, egraph);
                Constraint::All(vec![fc, gc])
            }
            _ => unsatisfied()
        }
    }

    let mut map = HashMap::new();
    let mut equivs = HashSet::new();
    for (ia, a) in egraph.classes().enumerate() {
        for b in egraph.classes().skip(ia + 1) {
            // println!("taking care of:\n{}\n and\n{}", a.data.beta_extract, b.data.beta_extract);
            eclasses(a.id, b.id, &mut map, &mut equivs, egraph);
        }
    }
    let total_size = egraph.total_size();
    let node_count = egraph.total_number_of_nodes();
    println!("total size: {}, node count: {}, alpha-equivalences: {}",
             total_size, node_count, equivs.len());
    equivs.len()
}

#[cfg(test)]
mod tests {
    use crate::rise::*;
    use crate::alpha_equiv::count_alpha_equiv;

    fn assert_alpha_equiv_count(n: usize, es: &[&str]) {
        let mut egraph = RiseEGraph::default();
        es.iter().for_each(|e| { egraph.add_expr(&e.parse().unwrap()); });
        egraph.rebuild();
        assert_eq!(count_alpha_equiv(&mut egraph), n);
    }

    #[test]
    fn alpha_equiv_count() {
        assert_alpha_equiv_count(0, &[
            "(lam y (lam x (app (app add (var x)) (var y)))))",
            "(lam x (lam y (app (app add (var x)) (var y)))))"
        ]);
        assert_alpha_equiv_count(2, &[
            "(lam y (lam x (app (app add (var x)) (var y)))))",
            "(lam x (lam y (app (app add (var y)) (var x)))))"
        ]);
        assert_alpha_equiv_count(0, &[
            "(lam y (lam x (app (app (app add (var free)) (var x)) (var y)))))",
            "(lam x (lam y (app (app (app add (var free)) (var x)) (var y)))))"
        ]);
        assert_alpha_equiv_count(2, &[
            "(lam y (lam x (app (app (app add (var free)) (var x)) (var y)))))",
            "(lam x (lam y (app (app (app add (var free)) (var y)) (var x)))))"
        ]);
    }
}
