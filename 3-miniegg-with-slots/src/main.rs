use miniegg_with_slots::*;

fn main() {
    let extraction = std::env::args().nth(2).as_deref() == Some("--extraction");

    match &*std::env::args().nth(1).unwrap() {
        "reduction" => test_reduction(extraction),
        "fission" => test_fission(extraction),
        "binomial" => test_binomial(extraction),
        _ => panic!(),
    }
}
