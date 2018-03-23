#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![allow(unused_imports)]
#![allow(unused_io_amount)]
extern crate sha1;
extern crate pretty_env_logger;
#[macro_use] extern crate log;

extern crate adler32;
extern crate crc32c_hw;


mod file;

use file::{FileTable};
// use std::convert::AsRef;

fn main() {
    pretty_env_logger::init();
    info!("start");
    let mut table = FileTable::new();
    table.scan("test");
    println!("{:?}", table);

}
