mod normalize;
mod step;
mod parse;
mod display;

type HashSet<T> = std::collections::BTreeSet<T>;
type HashMap<K, V> = std::collections::BTreeMap<K, V>;

#[derive(Clone)]
pub enum Ast {
    Lam(String, Box<Ast>),
    App(Box<Ast>, Box<Ast>),
    Var(String),
}

