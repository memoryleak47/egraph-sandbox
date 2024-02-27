use crate::*;

///// parse

pub fn parse(s: &str) -> RecExpr {
    let (ast, s) = parse_ast(s);
    assert!(s.is_empty());

    let mut re = RecExpr::new();
    let (_, _) = translate(ast, &mut re);

    re
}

// adds the ENode corresponding to `ast` to `re`, and returns its `AppliedId`.
// each free variable in `ast` corresponds to a Slot in the returned HashMap.
fn translate(ast: Ast, re: &mut RecExpr) -> (AppliedId, HashMap<String, Slot>) {
    match ast {
        Ast::Lam(x, b) => {
            let (b, mut map) = translate(*b, re);

            match map.remove(&x) {
                Some(x_slot) => {
                    let slot = Slot::fresh();

                    let mut slotmap = SlotMap::identity(&b.slots());
                    slotmap.insert(x_slot, slot);

                    let id = re.push(ENode::Lam(slot, b));
                    (id, map)
                },
                None => {
                    let slot = Slot::fresh();
                    let id = re.push(ENode::Lam(slot, b));
                    (id, map)
                },
            }
        },
        Ast::App(l, r) => todo!(),
        Ast::Var(x) => {
            let s = Slot::fresh();
            let id = re.push(ENode::Var(s));
            let mut map = HashMap::new();
            map.insert(x, s);

            (id, map)
        },
    }
}

///// to_string

fn to_ast(re: RecExpr) -> Ast {
    todo!()
}

pub fn to_string(re: RecExpr) -> String {
    let ast = to_ast(re);
    ast_to_string(ast)
}

#[test]
fn test_parse_roundtrip() {
    let s1 = "(app (lam x x) (lam y y))";
    let s2 = to_string(parse(s1));
    assert_eq!(s1, s2);
}
