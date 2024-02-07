#![feature(box_patterns)]

mod lang;
pub use lang::*;

mod tokenize;
pub use tokenize::*;

mod parser;
pub use parser::*;

mod run;
pub use run::*;

mod egg_compat;
pub use egg_compat::*;

mod eggtest;

use std::fs::*;
use std::io::Read;

fn main() {
    eggtest::main();
}

fn main2() {
    let filename: String = std::env::args().nth(1).unwrap_or(String::from("file.l"));
    let mut f = File::open(filename).unwrap();
    let mut data = String::new();
    f.read_to_string(&mut data).unwrap();

    let l = parse(data);
    let l = run(l);
    println!("{}", l);
}
