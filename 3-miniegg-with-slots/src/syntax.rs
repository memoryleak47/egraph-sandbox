use crate::*;

///// parse

pub fn parse(s: &str) -> RecExpr {
    let (ast, s) = parse_ast(s);
    assert!(s.is_empty());

    let mut re = RecExpr::new();
    translate(ast, &mut re, &HashMap::new());

    return re;
}

fn translate(ast: Ast, re: &mut RecExpr, vars: &HashMap<String, Slot>) -> AppliedId {
    match ast {
        Ast::Lam(x, b) => {
            let mut vars = vars.clone();
            let slot = Slot::fresh();
            vars.insert(x, slot);
            let b = translate(*b, re, &vars);
            re.push(ENode::Lam(slot, b))
        },
        Ast::App(l, r) => todo!(),
        Ast::Var(x) => todo!(),
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
