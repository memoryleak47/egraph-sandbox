#[derive(Clone)]
struct Slot(usize);

#[derive(Clone)]
struct AppliedId;

trait Language {
    type ENode<R>;

    fn get_rs<R>(n: &mut Self::ENode<R>) -> Vec<&mut R>;
    fn get_slots<R>(n: &mut Self::ENode<R>) -> Vec<&mut Slot>;
    fn map_rs<R1, R2>(n: &Self::ENode<R1>, f: impl Fn(&R1) -> R2) -> Self::ENode<R2>;
}

#[derive(Clone)]
struct Term<L: Language>(L::ENode<Box<Term<L>>>);
type Node<L: Language> = L::ENode<AppliedId>;

// example impl.
#[derive(Clone)]
enum MyL<R> {
    Number(i32),
    Plus(R, R),
}

impl Language for MyL<()> {
    type ENode<R> = MyL<R>;

    fn get_rs<R>(n: &mut Self::ENode<R>) -> Vec<&mut R> {
        match n {
            MyL::Number(_) => vec![],
            MyL::Plus(x, y) => vec![x, y],
        }
    }

    fn get_slots<R>(n: &mut Self::ENode<R>) -> Vec<&mut Slot> {
        match n {
            MyL::Number(_) => vec![],
            MyL::Plus(_, _) => vec![],
        }
    }

    fn map_rs<R1, R2>(n: &Self::ENode<R1>, f: impl Fn(&R1) -> R2) -> Self::ENode<R2> {
        match n {
            MyL::Number(i) => MyL::Number(*i),
            MyL::Plus(x, y) => MyL::Plus(f(x), f(y)),
        }
    }
}

fn main() {
    let x1: Term<MyL<()>> = Term(MyL::Number(23));
    let x2: Term<MyL<()>> = Term(MyL::Number(24));
    let x3: Term<MyL<()>> = Term(MyL::Plus(Box::new(x1), Box::new(x2)));
    // TODO: let x4 = x3.clone();
}
