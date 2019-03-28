extern crate compiletest_rs as compiletest;

use std::path::PathBuf;

fn run_mode(mode: &'static str) {

    let mut config = compiletest::Config::default();
    config.mode = "compile-fail".parse().expect("Invalid mode");
    config.rustc_path = PathBuf::from("target/debug/rure");
    config.src_base = PathBuf::from(format!("tests/{}", mode));
    config.runtool = Some("echo \"\" || ".to_owned());


    compiletest::run_tests(&config);
}

#[test]
fn compile_test() {
    run_mode("rure-fail");
    run_mode("rure-succ");
}
