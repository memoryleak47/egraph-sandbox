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

/*
fn to_ast(en: &ENode, name_map: HashMap<Slot, String>, namegen: &mut impl FnMut() -> String, v: &mut Vec<AstNode>) -> AstId {
    let n = re.last().unwrap();

    let enode = match n {
        ENode::Lam(x, b) => {
            let xname = namegen();
            let mut sub_name_map = name_map.clone();
            sub_name_map.insert(*x, xname.clone());
            sub_name_map = sub_name_map.into_iter().map(|(x, y)| (b.m.inverse()[x], y)).collect();

            let b = to_ast(&re[0..b.id.0+1], sub_name_map, namegen, v);

            AstNode::Lam(xname, b)
        },
        ENode::App(l, r) => {
            let l = to_ast(&re[0..l.id.0+1], name_map.clone(), namegen);
            let r = to_ast(&re[0..r.id.0+1], name_map, namegen);

            AstNode::App(Box::new(l), Box::new(r))
        },
        ENode::Var(x) => {
            let name = name_map[&x].clone();
            AstNode::Var(name)
        },
    };

    let idx = v.len();
    v.push(enode);
    idx
}
*/

pub fn to_string(re: RecExpr) -> String {
    let mut name_id = 0;
    let mut namegen = || {
        let name = format!("x{name_id}");
        name_id += 1;

        name
    };

    let mut v = Vec::new();
    for x in re.node_dag {
        // to_ast(&re.node_dag, Default::default(), &mut namegen, &mut v);
        // TODO
    }
    ast_to_string(v)
}

#[test]
fn test_parse_roundtrip() {
    let s1 = "(app (lam x0 x0) (lam x1 x1))";
    let (p, m) = parse(s1);
    let s2 = to_string(p, m);
    assert_eq!(s1, s2);
}
