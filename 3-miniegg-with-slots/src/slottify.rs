use crate::*;

pub fn slottify(s: &str) -> (String, HashMap<String, Slot>) {
    let mut s: String = s.replace("\n", " ").replace("\t", " ").replace("\r", " ");
    loop {
        let len = s.len();
        s = s.replace("  ", " ");
        s = s.replace("( ", "(");
        if len == s.len() { break; }
    }

    let matches1 = find_matches(&s, "(var ");
    let matches2 = find_matches(&s, "(lam ");
    let set: HashSet<_> = matches1.into_iter().chain(matches2.into_iter()).collect();

    let mut map = HashMap::default();
    for x in set {
        let slot = Slot::fresh();
        map.insert(x.clone(), slot);
        s = sound_replace(s, x, slot.to_string());
    }

    (s, map)
}

// returns all things that came directly after "pat", and are stopped by " ", ")" or "(".
fn find_matches(s: &str, pat: &str) -> Vec<String> {
    let mut out = Vec::new();

    let mut i = 0;
    while let Some(new_i) = s[i..].find(pat) {
        let sub = s[i+new_i+pat.len()..].chars().take_while(|x| ![' ', '(', ')'].contains(x)).collect();
        out.push(sub);
        i = i + new_i + 1;
    }

    out
}

// biggest hack of my life.
fn sound_replace(s: String, a: String, b: String) -> String {
    let mut s = s;
    let neighbour = [" ", "(", ")"];
    for n1 in neighbour {
        for n2 in neighbour {
            s = s.replace(&format!("{n1}{a}{n2}"), &format!("{n1}{b}{n2}"));
        }
    }
    s
}
