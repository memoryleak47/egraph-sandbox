use crate::*;

use std::str::FromStr;

define_language! {
pub enum DBRise {
    // lambda calculus:
    Lam(AppliedId) = "lam",
    App(AppliedId, AppliedId) = "app",
    Var(Index),
    Sigma(AppliedId, AppliedId, AppliedId) = "sig",
    Phi(AppliedId, AppliedId, AppliedId) = "phi",
    // can do that, but unfair to egg? :
    // Sigma(Index, AppliedId, AppliedId),
    // Phi(Index, Index, AppliedId),

    // rest:
    Number(i32),
    Symbol(Symbol),
}
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Copy)]
pub struct Index(pub u32);

bare_language_child!(Index);

impl std::fmt::Display for Index {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "%{}", self.0)
    }
}

impl FromStr for Index {
    type Err = Option<std::num::ParseIntError>;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if s.starts_with("%") {
            s["%".len()..].parse().map(Index).map_err(Some)
        } else {
            Err(None)
        }
    }
}
