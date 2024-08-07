use crate::*;
use crate::i_arith::build::*;

fn assert_reaches(start: RecExpr<ArithENode>, goal: RecExpr<ArithENode>, steps: usize) {
    let mut eg = EGraph::new();
    let i1 = eg.add_expr(start);
    for _ in 0..steps {
        rewrite_arith(&mut eg);
        if let Some(i2) = lookup_rec_expr(&goal, &eg) {
            if eg.eq(&i1, &i2) {
                return;
            }
        }
    }

    eg.dump();

    dbg!(extract::<_, AstSizeNoLet>(i1, &eg));
    dbg!(&goal);
    assert!(false);
}


#[test]
fn arith_test1() { // x+y = y+x
    let x = 0;
    let y = 1;

    let a = add2(var(x), var(y));
    let a = pattern_to_re(&a);

    let b = add2(var(y), var(x));
    let b = pattern_to_re(&b);

    assert_reaches(a, b, 3);
}

#[test]
fn arith_test2() { // (x+y) * (x+y) = (x+y) * (y+x)
    let x = 0;
    let y = 1;
    let z = 2;

    let a = mul2(
                add2(var(x), var(y)),
                add2(var(x), var(y))
            );
    let a = pattern_to_re(&a);

    let b = mul2(
                add2(var(x), var(y)),
                add2(var(y), var(x))
            );
    let b = pattern_to_re(&b);

    assert_reaches(a, b, 3);
}

#[test]
fn arith_test3() { // (x+y) * (y+z) = (z+y) * (y+x)
    let x = 0;
    let y = 1;
    let z = 2;

    let a = mul2(
                add2(var(x), var(y)),
                add2(var(y), var(z))
            );
    let a = pattern_to_re(&a);

    let b = mul2(
                add2(var(z), var(y)),
                add2(var(y), var(x))
            );
    let b = pattern_to_re(&b);

    assert_reaches(a, b, 3);
}

#[test]
fn arith_test4() { // (x+y)**2 = x**2 + x*y + x*y + y**2
    let x = 0;
    let y = 1;
    let z = 2;

    let a = mul2(
                add2(var(x), var(y)),
                add2(var(x), var(y))
            );
    let a = pattern_to_re(&a);

    let b = add2(
                mul2(var(x), var(x)),
                add2(
                    mul2(var(x), var(y)),
                    add2(
                        mul2(var(x), var(y)),
                        mul2(var(y), var(y)),
                    ),
                )
            );
    let b = pattern_to_re(&b);

    assert_reaches(a, b, 10);
}

fn add_chain(it: impl Iterator<Item=usize>) -> Pattern<ArithENode> {
    let mut it = it.map(var);
    let mut x = it.next().unwrap();
    for y in it {
        x = add2(x, y);
    }
    x
}

#[test]
fn arith_test5() { // x0+...+xN = xN+...+x0
    // This times out for larger N!
    const N: usize = 7;

    let a = add_chain(0..=N);
    let a = pattern_to_re(&a);

    let b = add_chain((0..=N).rev());
    let b = pattern_to_re(&b);

    assert_reaches(a, b, 10);
}
