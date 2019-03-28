use std::env;

extern crate rure;

fn main() {
    let args:Vec<String> = env::args().collect();

    rure::run(args);
}
