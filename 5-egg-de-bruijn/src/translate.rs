use crate::*;

// maps between the named & De Bruijn versions.

pub fn named_to_de_bruijn(s: &str) -> String {
    let ast = Ast::parse(s);

    ast_to_de_bruijn_str(&ast, &Default::default())
}

fn ast_to_de_bruijn_str(ast: &Ast, names: &HashMap<String, u32>) -> String {
    match ast {
        Ast::Var(x) => names[&*x].to_string(),
        Ast::Lam(x, b) => {
            let mut names: HashMap<String, u32> = names.iter().map(|(x, y)| (x.to_string(), y+1)).collect();
            names.insert(String::from(x), 0);
            let b = ast_to_de_bruijn_str(b, &names);
            format!("(lam {b})")
        },
        Ast::App(l, r) => {
            let l = ast_to_de_bruijn_str(&*l, names);
            let r = ast_to_de_bruijn_str(&*r, names);

            format!("(app {l} {r})")
        },
    }
}

pub fn de_bruijn_to_named(s: &str) -> String {
    let s: RecExpr<ENode> = s.parse().unwrap();

    de_bruijn_to_named_impl(s.as_ref().len()-1, s.as_ref(), &Default::default(), &mut 0)
}

fn de_bruijn_to_named_impl(i: usize, re: &[ENode], map: &HashMap<u32, String>, counter: &mut u32) -> String {
    match re[i] {
        ENode::App([l, r]) => {
            let l = de_bruijn_to_named_impl(usize::from(l), re, map, counter);
            let r = de_bruijn_to_named_impl(usize::from(r), re, map, counter);
            format!("(app {l} {r})")
        },
        ENode::Lam(b) => {
            let mut map: HashMap<u32, String> = map.iter().map(|(x, y)| (x+1, String::from(y))).collect();

            let x = format!("x{}", *counter);
            *counter += 1;

            map.insert(0, x.clone());

            let b = de_bruijn_to_named_impl(usize::from(b), re, &map, counter);

            format!("(lam {x} {b})")
        },
        ENode::Var(j) => map[&j].clone(),
        ENode::Placeholder(_) => panic!(),
    }
}
