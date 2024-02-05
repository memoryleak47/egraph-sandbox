use crate::lang::*;

#[derive(Debug)]
pub enum Token {
    Lambda, Dot, // for `\x. x`
    Ident(Var),
    LParen,
    RParen,
}

pub fn tokenize(s: String) -> Vec<Token> {
    let cs = s.chars().collect::<Vec<_>>();

    let mut tokens = Vec::new();
    let mut current_ident: Option<String> = None;
    for c in cs {
        if c.is_alphabetic() {
            if current_ident.is_none() { current_ident = Some(String::new()); }
            current_ident.as_mut().unwrap().push(c);
        } else {
            if let Some(i) = current_ident.take() {
                tokens.push(Token::Ident(i));
            }

            let t = match c {
                '\\' => Token::Lambda,
                '.' => Token::Dot,
                '(' => Token::LParen,
                ')' => Token::RParen,
                c if c.is_whitespace() => continue,
                _ => panic!("invalid char '{}'", c),
            };
            tokens.push(t);
        }
    }

    if let Some(i) = current_ident.take() {
        tokens.push(Token::Ident(i));
    }

    tokens
}
