use crate::*;

fn rand() -> usize {
    rand::random::<usize>()
}

fn any<T>(set: &HashSet<T>) -> &T {
    let r = rand() % set.len();
    set.iter().nth(r).unwrap()
}

fn binder_name() -> String {
    let mut hs = HashSet::default();
    for x in ["x", "y", "z", "a", "b", "c"] {
        hs.insert(String::from(x));
    }
    any(&hs).clone()
}

pub fn generate(cost: usize) -> Ast {
    generate_impl(cost, &HashSet::new())
}

fn generate_impl(cost: usize, declared: &HashSet<String>) -> Ast {
    #[derive(PartialOrd, Ord, PartialEq, Eq)]
    enum Opts { Var, Lam, App }

    let mut opts = HashSet::default();

    if !declared.is_empty() && cost <= 1 {
        opts.insert(Opts::Var);
    }

    if cost >= 2 {
        opts.insert(Opts::Lam);
    }

    if cost >= 3 {
        opts.insert(Opts::App);
    }

    if cost == 0 {
        if declared.is_empty() {
            opts.insert(Opts::Lam);
        } else {
            opts.insert(Opts::Var);
        }
    }

    let opt = any(&opts);

    match opt {
        Opts::Var => {
            let var = any(&declared).clone();
            Ast::Var(var)
        },
        Opts::Lam => {
            let n = binder_name();
            let mut decl = declared.clone();
            decl.insert(n.clone());
            let next = generate_impl(cost-1, &decl);

            Ast::Lam(n, Box::new(next))
        },
        Opts::App => {
            let scost = cost-1;
            let l_cost = rand() % scost;
            let r_cost = scost - l_cost;
            let l = generate_impl(l_cost, &declared);
            let r = generate_impl(r_cost, &declared);
            Ast::App(Box::new(l), Box::new(r))
        },
    }
}
