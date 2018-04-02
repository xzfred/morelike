#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![allow(unused_imports)]
extern crate pretty_env_logger;
extern crate sha1;
#[macro_use]
extern crate log;

extern crate adler32;
extern crate chrono;
extern crate console;
extern crate crc32c_hw;
extern crate indicatif;
extern crate time;

mod file;

use file::FileTable;
// use std::convert::AsRef;

fn main() {
    pretty_env_logger::init();
    info!("start");
    let mut table = FileTable::new();
    let p = "/Users/xuzhi/Music";
    table.scan(p);
    //table.scan("test");
    println!("{}", table);

    // println!("{:?}", table);

    table.exact();
}
