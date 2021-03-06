#![feature(rustc_private)]
#![feature(type_alias_enum_variants)]
extern crate rustc_driver;
extern crate rustc;
extern crate rustc_interface;
extern crate rustc_mir;
extern crate syntax_pos;
#[macro_use] extern crate text_io;

mod exec;

use exec::ExecutionConfig;

pub fn run(args: Vec<String>) {
    let ExecutionConfig = ExecutionConfig::new();

    ExecutionConfig.run(args);
}
