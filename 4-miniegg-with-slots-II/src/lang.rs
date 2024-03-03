use crate::*;

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id(pub usize);

pub type Slot = usize; // In each ENode, these form an interval [0..N].
                       // This corresponds to the public API of an ENode / EClass.
pub type Redundant = usize; // In each ENode, these form a different interval [0..N].
                            // They can be replaced by any Terms. They are "wildcards".

// TODO maybe have a version of PluggedId with only Plug::Slot for user-facing stuff?
// Would also be useful for the unionfind datastructure.
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct PluggedId {
    pub id: Id,
    pub plugs: Vec<Plug>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Plug {
    Slot(Slot),
    Lam,
    Redundant(Redundant),
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum ENode {
    Lam(PluggedId),
    App(PluggedId, PluggedId),
    Var, // always uses Slot 0.
}

#[derive(Clone, Debug)]
pub struct RecExpr {
    pub node_dag: Vec<ENode>,
}

impl ENode {
    // lists all plugs in order of their usages (not deduplicated).
    pub fn plugs(&self) -> Vec<Plug> {
        let mut v = Vec::new();
        match self {
            ENode::Lam(r) => {
                v.extend(r.plugs.clone());
            },
            ENode::App(l, r) => {
                v.extend(l.plugs.clone());
                v.extend(r.plugs.clone());
            }
            ENode::Var => {
                v.push(Plug::Slot(0));
            },
        };

        v
    }

    pub fn map_plugs(&self, f: impl Fn(Plug) -> Plug) -> ENode {
        match self {
            ENode::Lam(r) => ENode::Lam(r.map_plugs(f)),
            ENode::App(l, r) => ENode::App(l.map_plugs(&f), r.map_plugs(f)),
            ENode::Var => ENode::Var,
        }
    }

    // sorts the redundant slots to be ordered by usage.
    pub fn with_sorted_redundants(&self) -> ENode {
        let mut redundants = HashMap::new();
        for x in self.plugs() {
            if let Plug::Redundant(i) = x {
                if !redundants.contains_key(&i) {
                    redundants.insert(i, redundants.len());
                }
            }
        }

        self.map_plugs(|x| {
            match x {
                Plug::Redundant(i) => Plug::Redundant(redundants[&i]),
                y => y,
            }
        })
    }
}

impl PluggedId {
    fn map_plugs(&self, f: impl Fn(Plug) -> Plug) -> Self {
        Self {
            id: self.id,
            plugs: self.plugs.iter().cloned().map(f).collect(),
        }
    }
}
