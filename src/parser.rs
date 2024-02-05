use crate::lang::*;
use crate::tokenize::*;

pub fn parse(s: String) -> Term {
    let tokens = tokenize(s);
    let (tokens, t) = parse_term(&tokens[..]);
    assert!(tokens.is_empty());

    t
}

fn parse_term(mut tokens: &[Token]) -> (&[Token], Term) {
    let (toks, mut t) = parse_single_term(tokens);
    tokens = toks;

    while !tokens.is_empty() && tokens[0] != Token::RParen {
        let (toks, t2) = parse_single_term(tokens);
        tokens = toks;
        t = Term::Application(Box::new(t), Box::new(t2));
    }

    (tokens, t)
}

fn parse_single_term(tokens: &[Token]) -> (&[Token], Term) {
    match tokens {
        [Token::LParen, rest@..] => {
            let (toks, t) = parse_term(rest);
            assert_eq!(toks[0], Token::RParen);

            return (&toks[1..], t);
        },
        [Token::Lambda, Token::Ident(i), Token::Dot, rest@..] => {
            let (toks, t) = parse_term(rest);
            let t = Term::Abstraction(i.to_string(), Box::new(t));
            return (toks, t);
        },
        [Token::Ident(v), rest@..] => {
            (rest, Term::Var(v.to_string()))
        },
        _ => panic!(),
    }
}
