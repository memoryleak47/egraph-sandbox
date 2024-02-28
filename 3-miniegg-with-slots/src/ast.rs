use crate::*;

pub type AstId = usize;

#[derive(Debug)]
pub enum AstNode {
    Lam(String, AstId),
    App(AstId, AstId),
    Var(String),
}

pub fn parse_ast(s: &str) -> Vec<AstNode> {
    let mut v = Vec::new();
    let (_, s) = parse_ast_impl(s, &mut v);
    assert!(s.is_empty());

    v
}

fn parse_ast_impl<'s>(s: &'s str, v: &mut Vec<AstNode>) -> (AstId, &'s str) {
    if s.starts_with("(lam ") {
        let s = &s["(lam ".len()..];
        let (ident, s) = parse_ident(s);

        assert!(s.starts_with(" "));
        let s = &s[1..];

        let (b, s) = parse_ast_impl(s, v);
        let ident = ident.to_string();

        assert!(s.starts_with(")"));
        let s = &s[1..];

        let idx = v.len();
        v.push(AstNode::Lam(ident, b));

        (idx, s)
    } else if s.starts_with("(app ") {
        let s = &s["(app ".len()..];
        let (l, s) = parse_ast_impl(s, v);

        assert!(s.starts_with(" "));
        let s = &s[1..];

        let (r, s) = parse_ast_impl(s, v);

        assert!(s.starts_with(")"));
        let s = &s[1..];

        let idx = v.len();
        v.push(AstNode::App(l, r));

        (idx, s)
    } else {
        let (ident, s) = parse_ident(s);
        let idx = v.len();
        v.push(AstNode::Var(ident.to_string()));

        (idx, s)
    }
}

fn parse_ident(s: &str) -> (/*ident*/ &str, /*rest*/ &str) {
    let i = s.find(|x| x == '(' || x == ')' || x == ' ').unwrap_or(s.len());

    let ident = &s[0..i];
    let rest = &s[i..];

    (ident, rest)
}

pub fn ast_to_string(a: Vec<AstNode>) -> String {
    let mut strings = Vec::new();
    for n in a {
        let s = match n {
            AstNode::Lam(x, b) => format!("(lam {} {})", x, strings[b]),
            AstNode::App(l, r) => format!("(app {} {})", strings[l], strings[r]),
            AstNode::Var(x) => format!("{x}"),
        };
        strings.push(s);
    }

    strings.last().unwrap().to_string()
}
