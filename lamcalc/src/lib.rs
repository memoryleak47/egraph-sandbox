mod normalize;
pub use normalize::*;

mod step;
pub use step::*;

mod parse;
pub use parse::*;

mod display;
pub use display::*;

mod realization;
pub use realization::*;

mod build;
pub use build::*;

mod tst;
pub use tst::*;

type HashSet<T> = std::collections::BTreeSet<T>;
type HashMap<K, V> = std::collections::BTreeMap<K, V>;

#[derive(Clone)]
pub enum Ast {
    Lam(String, Box<Ast>),
    App(Box<Ast>, Box<Ast>),
    Var(String),
}

