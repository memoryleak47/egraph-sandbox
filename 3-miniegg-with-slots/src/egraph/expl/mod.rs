use crate::*;

mod proof;
pub use proof::*;

mod step;
pub use step::*;

impl<L: Language> EGraph<L> {
    pub fn explain_equivalence(&mut self, t1: RecExpr<L>, t2: RecExpr<L>) -> ProvenEq {
        let i1 = self.add_syn_expr(t1);
        let i2 = self.add_syn_expr(t2);

        if !self.eq(&i1, &i2) { panic!("Can't explain an equivalence that does not hold!"); }

        let (_, prf1) = self.proven_unionfind_get(i1.id);
        let (_, prf2) = self.proven_unionfind_get(i2.id);
        let prf2 = self.prove_symmetry(prf2);
        let p = self.prove_transitivity(prf1, prf2);

        if CHECKS {
            let eq1 = Equation { l: i1, r: i2 };
            match_equation(&eq1, &p).unwrap();
        }
        p
    }
}


type ShowMap = HashMap<*const ProvenEqRaw, (usize, String)>;

impl ProvenEqRaw {
    pub fn show(&self) {
        self.show_impl(&|i| format!("{i:?}"))
    }

    pub fn show_expr<L: Language>(&self, eg: &EGraph<L>) {
        self.show_impl(&|i| {
            eg.get_syn_expr(i).to_string()
        })
    }

    pub fn show_impl(&self, f: &impl Fn(&AppliedId) -> String) {
        let mut map = Default::default();
        self.show_impl2(&mut map, f);

        let mut map_sorted: Vec<_> = map.into_iter().collect();
        map_sorted.sort_by_key(|(_, (i, _))| *i);
        for (_, (_, s)) in map_sorted {
            println!("{}", s);
        }
    }

    pub fn show_impl2(&self, v: &mut ShowMap, f: &impl Fn(&AppliedId) -> String) {
        let ptr = (&*self) as *const ProvenEqRaw;
        if v.contains_key(&ptr) { return; }

        let id_of = |p: &ProvenEq, v: &mut ShowMap| -> usize {
            p.show_impl2(v, f);
            let ptr = (&**p) as *const ProvenEqRaw;
            v[&ptr].0
        };

        let prf = match self.proof() {
            Proof::Explicit(j) => format!("{j:?}"),
            Proof::Reflexivity => format!("refl"),
            Proof::Symmetry(x) => format!("symmetry({})", id_of(x, v)),
            Proof::Transitivity(x1, x2) => {
                let y1 = id_of(x1, v);
                let y2 = id_of(x2, v);
                format!("transitivity({}, {})", y1, y2)
            },
            Proof::Congruence(xs) => {
                let xs: Vec<_> = xs.into_iter().map(|x| id_of(x, v).to_string()).collect();
                let s = xs.join(", ");
                format!("congruence({s})")
            },

            Proof::Shrink(x) => format!("shrink({})", id_of(x, v))
        };

        let i = v.len();
        let Equation { l, r } = &**self;
        let out = format!("lemma{i}: '{} = {}'", f(l), f(r));
        let out = format!("{out}\n  by {prf}\n");
        v.insert(ptr, (i, out));
    }
}
