use crate::*;

// This module is recording all equations that happen during the usage of an e-graph.
// Everything relevant for this, is defined here.

impl<L: Language> Explain<L> {
    // translates an egraph e-class to its corresponding term id.
    pub fn translate(&self, l: &AppliedId) -> AppliedId {
        // l.m :: slots(l.id) -> X
        let a = &self.translator[&l.id];
        // a == l.id

        // a has some redundant slot choices.
        let mut m = l.m.clone();
        for s in a.slots() {
            if !m.contains_key(s) {
                m.insert(s, Slot::fresh());
            }
        }
        a.apply_slotmap(&m)
    }

    // translates an egraph e-node to its corresponding explain e-node.
    pub fn translate_enode(&self, e: &L) -> L {
        e.map_applied_ids(|x| self.translate(&x))
    }

    // Both l and i are in egraph world.
    pub fn add_translation(&mut self, l: L, i: AppliedId) {
        // l == i holds in the egraph world.
        let i2 = self.add_egraph_enode(l);
        // i should now translate to i2.
        
        // i == i2
        // i.id * i.m == i2
        // i.id == i2 * i.m^-1
        let i2_id = i2.apply_slotmap(&i.m.inverse());
        self.translator.insert(i.id, i2_id);
    }
 
    pub fn add_egraph_enode(&mut self, l: L) -> AppliedId {
        let l = self.translate_enode(&l);
        self.add_explain_enode(l)
    }

    // adds an e-node to the term-id <-> e-node bijection.
    // and returns the corresponding AppliedId.
    // Both input and output are completely in the explain-world.
    pub fn add_explain_enode(&mut self, l: L) -> AppliedId {
        let (sh, bij) = l.weak_shape();
        if let Some(x) = self.enode_to_term_id.get(&sh) {
            x.apply_slotmap(&bij)
        } else {
            let i = Id(self.enode_to_term_id.len());
            // i == l
            // -> i == sh * bij
            // -> sh == i * bij^-1
            let app_id = AppliedId::new(i, bij.inverse());
            self.enode_to_term_id.insert(sh, app_id);
            self.term_id_to_enode.insert(i, l.clone());
            let identity = bij.inverse().compose(&bij);
            AppliedId::new(i, identity)
        }
    }

    pub fn add_term(&mut self, t: &RecExpr<L>) -> AppliedId {
        let mut n = t.node.clone();
        let mut refs: Vec<&mut _> = n.applied_id_occurences_mut();
        for i in 0..refs.len() {
            *(refs[i]) = self.add_term(&t.children[i]);
        }
        self.add_explain_enode(n)
    }

    pub fn enode_to_term_id(&self, l: &L) -> Option<AppliedId> {
        let (sh, bij) = l.weak_shape();
        let a = self.enode_to_term_id.get(&sh)?;
        // a == sh by definition of a.
        // sh * bij == l by definition of (sh, bij).
        // -> a * bij == l
        Some(a.apply_slotmap(&bij))
    }

    pub fn term_id_to_enode(&self, a: &AppliedId) -> Option<L> {
        let x = self.term_id_to_enode.get(&a.id)?;
        // x == a.id by definition of x.
        // a == a.id * a.m by definition of AppliedId.
        // -> a == x * a.m
        let out = x.apply_slotmap(&a.m);
        let out = out.refresh_internals(out.slots());
        Some(out)
    }

    pub fn term_id_to_term(&self, a: &AppliedId) -> Option<RecExpr<L>> {
        let enode = self.term_id_to_enode(a)?;
        let cs = enode.applied_id_occurences()
                      .iter()
                      .map(|x| self.term_id_to_term(x).unwrap())
                      .collect();
        Some(RecExpr {
            node: nullify_app_ids(&enode),
            children: cs,
        })
    }

    // Both arguments are Explain AppliedIds.
    pub fn add_equation(&mut self, a: AppliedId, b: AppliedId, j: Justification) {
        let a_id = a.id;
        let b_id = b.id;

        let i = self.equations.len();
        let eq = Equation {
            l: a,
            r: b,
            j,
        };
        self.equations.push(eq);
    }

    // Subst contains Explain-AppliedIds.
    // This also returns an Explain-AppliedId.
    pub fn pattern_subst(&mut self, pat: &Pattern<L>, subst: &Subst) -> AppliedId {
        match &pat.node {
            ENodeOrPVar::ENode(n) => {
                let mut n = n.clone();
                let mut refs: Vec<&mut _> = n.applied_id_occurences_mut();
                assert_eq!(pat.children.len(), refs.len());
                for i in 0..refs.len() {
                    *(refs[i]) = self.pattern_subst(&pat.children[i], subst);
                }
                self.add_explain_enode(n)
            },
            ENodeOrPVar::PVar(v) => {
                subst[v].clone()
            },
        }
    }

    pub fn mk_identity_app_id(&self, i: Id) -> AppliedId {
        let slots = self.slots_of(i);
        let identity = SlotMap::identity(&slots);
        AppliedId::new(i, identity)
    }

    pub fn slots_of(&self, i: Id) -> HashSet<Slot> {
        self.term_id_to_enode[&i].slots()
    }
}
