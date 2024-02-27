use crate::*;

#[derive(Debug)]
pub enum Ast {
    Lam(String, Box<Ast>),
    App(Box<Ast>, Box<Ast>),
    Var(String),
}

pub fn parse_ast(s: &str) -> (Ast, &str) {
    if s.starts_with("(lam ") {
        let s = &s["(lam ".len()..];
        let (ident, s) = parse_ident(s);

        assert!(s.starts_with(" "));
        let s = &s[1..];

        let (b, s) = parse_ast(s);
        let ident = ident.to_string();

        assert!(s.starts_with(")"));
        let s = &s[1..];

        let ast = Ast::Lam(ident, Box::new(b));

        (ast, s)
    } else if s.starts_with("(app ") {
        let s = &s["(app ".len()..];
        let (l, s) = parse_ast(s);

        assert!(s.starts_with(" "));
        let s = &s[1..];

        let (r, s) = parse_ast(s);

        assert!(s.starts_with(")"));
        let s = &s[1..];

        let ast = Ast::App(Box::new(l), Box::new(r));

        (ast, s)
    } else {
        let (ident, s) = parse_ident(s);
        let ast = Ast::Var(ident.to_string());

        (ast, s)
    }
}

fn parse_ident(s: &str) -> (/*ident*/ &str, /*rest*/ &str) {
    let i = s.find(|x| x == '(' || x == ')' || x == ' ').unwrap_or(s.len());

    let ident = &s[0..i];
    let rest = &s[i..];

    (ident, rest)
}

pub fn ast_to_string(a: Ast) -> String {
    match a {
        Ast::Lam(x, b) => format!("(lam {} {})", x, ast_to_string(*b)),
        Ast::App(l, r) => format!("(app {} {})", ast_to_string(*l), ast_to_string(*r)),
        Ast::Var(x) => format!("{x}"),
    }
}
