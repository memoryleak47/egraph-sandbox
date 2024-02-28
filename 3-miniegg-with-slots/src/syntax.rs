use crate::*;

///// parse

pub fn parse(s: &str) -> RecExpr {
    let ast = parse_ast(s);

    let mut v = Vec::new();
    for x in ast {
        let x_v = translate(x, &v);

        let name_map_slots: HashSet<_> = x_v.name_map.values().copied().collect();
        let enode_slots = x_v.enode.slots();

        v.push(x_v);

        if name_map_slots != enode_slots {
            dbg!(&v);
            println!("{:?}", name_map_slots);
            println!("!=");
            println!("{:?}", enode_slots);
            panic!("slots error!");
        }
    }

    assert!(v.last().unwrap().name_map.is_empty(), "Free variables are not allowed in parsed terms!");

    let node_dag = v.into_iter().map(|x| x.enode).collect();

    RecExpr { node_dag }
}

#[derive(Clone, Debug)]
struct TranslateData {
    enode: ENode,
    name_map: HashMap<String, Slot>,
}

// for (a: AppliedId, m: HashMap<..>) = translate(..); we require a.slots() == m.values();
fn translate(ast_node: AstNode, v: &[TranslateData]) -> TranslateData {
    match ast_node {
        AstNode::Lam(x, b) => {
            let b_data = v[b].clone();

            // The slot in the ENode::Lam(..) that we will create.
            let lam_slot = Slot::fresh();

            let mut slotmap = SlotMap::new();
            let mut name_map = HashMap::new();

            if let Some(xb_slot) = b_data.name_map.get(&x) {
                slotmap.insert(*xb_slot, lam_slot);
            }

            for (name, &s) in &b_data.name_map {
                if name == &x { continue; }

                let new_s = Slot::fresh();
                if !slotmap.contains_key(s) {
                    slotmap.insert(s, new_s);
                }
                name_map.insert(name.to_string(), new_s);
            }

            let id = AppliedId::new(Id(b), slotmap);
            let enode = ENode::Lam(lam_slot, id);

            TranslateData { enode, name_map }
        },
        AstNode::App(l, r) => todo!(),
        AstNode::Var(x) => {
            let s = Slot::fresh();
            let enode = ENode::Var(s);
            let mut name_map = HashMap::new();
            name_map.insert(x, s);

            TranslateData { enode, name_map }
        },
    }
}

///// to_string

pub fn to_string(re: RecExpr) -> String {
    let mut name_id = 0;
    let mut namegen = || {
        let name = format!("x{name_id}");
        name_id += 1;

        name
    };

    let en: ENode = re.node_dag.last().unwrap().clone();

    to_string_impl(en, &re.node_dag, Default::default(), &mut namegen)
}

fn to_string_impl(en: ENode, re: &[ENode], name_map: HashMap<Slot, String>, namegen: &mut impl FnMut() -> String) -> String {
    match en {
        ENode::Lam(x, b) => {
            let xname = namegen();
            let b_node = re[b.id.0].clone();
            let m = b.m.inverse();
            let mut name_map: HashMap<_, _> = name_map.iter().map(|(x, y)| (m[*x], y.clone())).collect();
            if m.contains_key(x) {
                name_map.insert(m[x], xname.clone());
            }
            let b = to_string_impl(b_node, re, name_map, namegen);
            format!("(lam {xname} {b})")
        },
        ENode::App(l, r) => {
            let mut call = |a: AppliedId| {
                let node = re[a.id.0].clone();
                let m = a.m.clone();
                let a_name_map: HashMap<_, _> = node.slots().iter().map(|x| (*x, name_map[&m[*x]].clone())).collect();

                to_string_impl(node, re, a_name_map, namegen)
            };
            let l = call(l);
            let r = call(r);
            format!("(app {l} {r})")
        },
        ENode::Var(x) => {
            let name = name_map.get(&x)
                               .unwrap_or_else(|| panic!("Free variables are forbidden in `to_string`!"));
            format!("{name}")
        },
    }
}

#[test]
fn test_parse_roundtrip() {
    let s1 = "(app (lam x0 x0) (lam x1 x1))";
    let p = parse(s1);
    let s2 = to_string(p);
    assert_eq!(s1, s2);
}
