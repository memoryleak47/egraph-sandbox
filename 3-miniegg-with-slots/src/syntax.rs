use crate::*;

pub fn parse(s: &str) -> RecExpr {
    let mut node_dag = vec![];

    let s = parse_impl(s, &mut node_dag);
    assert!(s.is_empty());

    RecExpr { node_dag }
}

fn parse_impl<'a>(s: &'a str, node_dag: &mut Vec<ENode>) -> &'a str {
    if s.starts_with("(lam ") {
        let s = &s["(lam ".len()..];
        let (ident, s) = parse_ident(s);

        assert!(s.starts_with(" "));
        let s = &s[1..];

        let s = parse_impl(s, node_dag);

        let ident = ident.to_string();
        let id = Id(node_dag.len() - 1);

        assert!(s.starts_with(")"));
        let s = &s[1..];

        node_dag.push(ENode::Lam(ident, id));

        s
    } else if s.starts_with("(app ") {
        let s = &s["(app ".len()..];
        let s = parse_impl(s, node_dag);
        let id1 = Id(node_dag.len() - 1);

        assert!(s.starts_with(" "));
        let s = &s[1..];

        let s = parse_impl(s, node_dag);
        let id2 = Id(node_dag.len() - 1);

        assert!(s.starts_with(")"));
        let s = &s[1..];

        node_dag.push(ENode::App(id1, id2));

        s
    } else {
        let (ident, s) = parse_ident(s);
        node_dag.push(ENode::Var(ident.to_string()));

        s
    }
}

fn parse_ident(s: &str) -> (/*ident*/ &str, /*rest*/ &str) {
    let i = s.find(|x| x == '(' || x == ')' || x == ' ').unwrap();

    let ident = &s[0..i];
    let rest = &s[i..];

    (ident, rest)
}

pub fn to_string(re: RecExpr) -> String {
    let mut strings = vec![];
    for x in re.node_dag.iter() {
        let s = match x {
            ENode::Lam(x, b) => format!("(lam {} {})", x, &strings[b.0]),
            ENode::App(l, r) => format!("(app {} {})", &strings[l.0], &strings[r.0]),
            ENode::Var(x) => format!("{x}"),
        };
        strings.push(s);
    }

    strings.pop().unwrap()
}

#[test]
fn test_parse_roundtrip() {
    let s1 = "(app (lam x x) (lam y y))";
    let s2 = to_string(parse(s1));
    assert_eq!(s1, s2);
}
