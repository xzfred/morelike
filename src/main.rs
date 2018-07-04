#[macro_use] extern crate log;
extern crate pretty_env_logger;

// extern crate adler32;
// extern crate chrono;
// extern crate console;
// extern crate crc32c_hw;
// extern crate sha1;
// extern crate typenum;
// extern crate digest;
// #[macro_use] extern crate generic_array;
// extern crate indicatif;
// extern crate time;
extern crate twox_hash;

// pub mod task;
// use task::{MsgPos, FinderMsg, Finder};
use std::{thread, time};
// use std::sync::{Arc};

mod finder;
mod taskpool;
mod sum;

fn main() {
    pretty_env_logger::init();
    debug!("start: ++++++++++++++++++++++");
    // finder::scan("/Users/xuzhi/my");

    // taskpool::ThreadPool::new().spawn(|| info!("i am thread!"));

    let comparer = sum::Comparer::new();
    //     // f.scan("/Users/xuzhi/my/dev/morelike");
    // comparer.run("/Users/fred/my/dev/morelike/test");
    comparer.run("/Users/fred/my/zip");

    let ten_millis = time::Duration::from_millis(1);
    thread::sleep(ten_millis);
    debug!("end: ++++++++++++++++++++++");
}
