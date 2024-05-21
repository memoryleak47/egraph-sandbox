use crate::*;
use crate::i_arith::build::*;

fn assert_reaches(start: RecExpr2<ArithENode>, goal: RecExpr2<ArithENode>, steps: usize) {
    let mut eg = EGraph::new();
    let i1 = add_rec_expr2(&start, &mut eg);
    for _ in 0..steps {
        rewrite_arith(&mut eg);
        if let Some(i2) = lookup_rec_expr2(&goal, &eg) {
            if eg.eq(&i1, &i2) {
                return;
            }
        }
    }

    eg.dump();

    dbg!(extract::<_, AstSizeNoLet>(i1.id, &eg));
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
    let z = 1;

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
    let z = 1;

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
    let z = 1;

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
