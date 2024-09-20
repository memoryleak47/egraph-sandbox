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

        let prf1 = self.unionfind_get_proof(i1.id);
        let prf2 = self.unionfind_get_proof(i2.id);
        let prf2 = self.prove_symmetry(prf2);
        let p = self.prove_transitivity(prf1, prf2);
        assert_eq!(&p.l, &i1);
        assert_eq!(&p.r, &i2);
        p
    }
}


type ShowMap = HashMap<*const ProvenEqRaw, (usize, String)>;

impl ProvenEqRaw {
    pub fn show(&self) {
        let mut map = Default::default();
        self.show_impl(&mut map);

        let mut map_sorted: Vec<_> = map.into_iter().collect();
        map_sorted.sort_by_key(|(_, (i, _))| *i);
        for (_, (_, s)) in map_sorted {
            println!("{}", s);
        }
    }

    pub fn show_impl(&self, v: &mut ShowMap) {
        let ptr = (&*self) as *const ProvenEqRaw;
        if v.contains_key(&ptr) { return; }

        let id_of = |p: &ProvenEq, v: &mut ShowMap| -> usize {
            p.show_impl(v);
            let ptr = (&**p) as *const ProvenEqRaw;
            v[&ptr].0
        };

        let Equation { l, r } = &**self;
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
        let out = format!("lemma{i}: {l:?} = {r:?}");
        let out = format!("{out}\n  by {prf}\n");
        v.insert(ptr, (i, out));
    }
}
