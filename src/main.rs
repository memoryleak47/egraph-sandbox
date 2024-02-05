mod lang;
pub use lang::*;

mod tokenize;
pub use tokenize::*;

use std::fs::*;
use std::io::Read;

fn main() {
    let filename: String = std::env::args().nth(1).unwrap_or(String::from("file.l"));
    let mut f = File::open(filename).unwrap();
    let mut data = String::new();
    f.read_to_string(&mut data).unwrap();

    dbg!(tokenize(data));
}
