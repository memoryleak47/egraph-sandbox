#![allow(unused)]

use crate::*;

// proof draft without congruence details.
#[derive(Debug)]
pub struct EqPath {
    pub start: AppliedId,
    pub end: AppliedId,
    pub elems: Vec<Equation>,
}

impl<L: Language> Explain<L> {
    pub fn find_path(&self, a: &AppliedId, b: &AppliedId) -> EqPath {
        let p = self.find_path_modulo_slots(a, b.id);

        assert_eq!(&p.start, a);
        assert_eq!(p.end.id, b.id);
        p.assert_valid();

        if &p.end != b {
            panic!("Slot mismatch! Explanations don't yet work for redundant slots & symmetries");
        }

        p
    }

    fn find_path_modulo_slots(&self, a: &AppliedId, b_id: Id) -> EqPath {
        if a.id == b_id {
            return EqPath {
                start: a.clone(),
                end: a.clone(),
                elems: Vec::new(),
            };
        }

        // maps each Id `r_id` to an `Equation(l, r, j)`,
        // where r_id = r.id and
        // l.id is a step closer to a.id.
        let mut back_eq: HashMap<Id, Equation> = HashMap::default();

        let mut open = HashSet::default();
        open.insert(a.id);

        // compute back_eq:
        while open.len() > 0 {
            let last_open = open;
            open = HashSet::default();

            for x in last_open {
                for &i in &self.imap[&x] {
                    let mut eq = self.equations[i].clone();

                    // flip x to be on the left-side of the equation.
                    if x != eq.l.id {
                        eq = eq.flip();
                    }
                    let l = eq.l.id;
                    let r = eq.r.id;
                    assert_eq!(x, l);

                    if !back_eq.contains_key(&r) && r != a.id {
                        back_eq.insert(r, eq);
                        open.insert(r);
                    }
                }
            }
        }

        assert!(back_eq.contains_key(&b_id));

        // compute eq_path:
        let mut eq_path = vec![];
        let mut i = b_id;
        while i != a.id {
            let beq = back_eq[&i].clone();
            i = beq.l.id;
            eq_path.push(beq);
        }
        eq_path.reverse();

        // compute path:
        let mut elems = Vec::new();
        let mut current = a.clone();
        for eq in &eq_path {
            let next = match_eq(&current, eq);
            let current_eq = Equation {
                l: current,
                r: next.clone(),
                j: eq.j.clone()
            };

            elems.push(current_eq);
            current = next;
        }

        EqPath {
            start: a.clone(),
            end: current,
            elems,
        }
    }
}

// x.m :: slots(l) -> X
// eq.l.m :: slots(l) -> Y
// eq.r.m :: slots(r) -> Y
// return.m :: slots(r) -> X
// -> return.m = eq.r.m * eq.l.m^-1 * x.m
fn match_eq(x: &AppliedId, eq: &Equation) -> AppliedId {
    let m = eq.r.m.compose_fresh(&eq.l.m.inverse())
                  .compose_fresh(&x.m);
    AppliedId::new(eq.r.id, m)
}

impl EqPath {
    fn assert_valid(&self) {
        if self.elems.len() > 0 {
            assert_eq!(self.start, self.elems.first().unwrap().l);
            assert_eq!(self.end, self.elems.last().unwrap().r);

            for i in 0..(self.elems.len()-1) {
                assert_eq!(self.elems[i].r, self.elems[i+1].l);
            }
        } else {
            assert_eq!(self.start, self.end);
        }
    }
}
