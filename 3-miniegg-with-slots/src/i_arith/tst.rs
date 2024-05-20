use crate::*;
use crate::i_arith::build::*;

fn assert_reaches(start: RecExpr2<ArithENode>, goal: RecExpr2<ArithENode>, steps: usize) {
    let mut eg = EGraph::new();
    let i1 = add_rec_expr2(&start, &mut eg);
    for _ in 0..steps {
        rewrite_arith(&mut eg);
        if let Some(i2) = lookup_rec_expr2(&goal, &eg) {
            if eg.find_id(i1.id) == eg.find_id(i2.id) {
                return;
            }
        }
    }

    dbg!(extract::<_, AstSizeNoLet>(i1.id, &eg));
    dbg!(&goal);
    assert!(false);
}

