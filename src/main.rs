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

mod task;
use task::Task;

fn main() {
    pretty_env_logger::init();
    info!("start");
    // let p = "/Users/xuzhi/Music";
    let p = "test";
    let mut file_task = Task::new();
    file_task.scan(p);

    println!("{:?}", file_task);

    // let mut table = FileTable::new();
    // table.scan(p);
    // //table.scan("test");
    // println!("{}", table);

    // // println!("{:?}", table);
    // table.exact();
}
