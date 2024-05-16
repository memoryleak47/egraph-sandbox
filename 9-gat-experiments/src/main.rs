#[derive(Clone)]
struct Slot(usize);

#[derive(Clone)]
struct AppliedId;

trait Language {
    type ENode<R: Clone>: Clone;

    fn get_rs<R: Clone>(n: &mut Self::ENode<R>) -> Vec<&mut R>;
    fn get_slots<R: Clone>(n: &mut Self::ENode<R>) -> Vec<&mut Slot>;
    fn map_rs<R1: Clone, R2: Clone>(n: &Self::ENode<R1>, f: impl Fn(&R1) -> R2) -> Self::ENode<R2>;
}

struct Term<L: Language>(L::ENode<Box<Term<L>>>);
type Node<L: Language> = L::ENode<AppliedId>;

impl<L: Language> Clone for Term<L> {
    fn clone(&self) -> Self {
        Term(self.0.clone())
    }
}

struct MyLang;

// example impl.
#[derive(Clone)]
enum MyL<R: Clone> {
    Number(i32),
    Plus(R, R),
}

impl Language for MyLang {
    type ENode<R: Clone> = MyL<R>;

    fn get_rs<R: Clone>(n: &mut Self::ENode<R>) -> Vec<&mut R> {
        match n {
            MyL::Number(_) => vec![],
            MyL::Plus(x, y) => vec![x, y],
        }
    }

    fn get_slots<R: Clone>(n: &mut Self::ENode<R>) -> Vec<&mut Slot> {
        match n {
            MyL::Number(_) => vec![],
            MyL::Plus(_, _) => vec![],
        }
    }

    fn map_rs<R1: Clone, R2: Clone>(n: &Self::ENode<R1>, f: impl Fn(&R1) -> R2) -> Self::ENode<R2> {
        match n {
            MyL::Number(i) => MyL::Number(*i),
            MyL::Plus(x, y) => MyL::Plus(f(x), f(y)),
        }
    }
}

fn main() {
    let x1: Term<MyLang> = Term(MyL::Number(23));
    let x2: Term<MyLang> = Term(MyL::Number(24));
    let x3: Term<MyLang> = Term(MyL::Plus(Box::new(x1), Box::new(x2)));
    let x4 = x3.clone();
}
