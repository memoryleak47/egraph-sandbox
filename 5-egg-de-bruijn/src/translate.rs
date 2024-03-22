use crate::*;

// maps between the AST & RecExpr using De Bruijn versions.

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
