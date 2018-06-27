#[macro_use]
extern crate log;
extern crate pretty_env_logger;

extern crate adler32;
extern crate chrono;
extern crate console;
// extern crate crc32c_hw;
extern crate indicatif;
// extern crate time;

// pub mod task;
// use task::{MsgPos, FinderMsg, Finder};
use std::{thread, time};
// use std::sync::{Arc};

mod finder;
mod taskpool;
// use finder::*;

fn main() {
    pretty_env_logger::init();
    debug!("start: ++++++++++++++++++++++");
    // finder::scan("/Users/xuzhi/my");

    // taskpool::ThreadPool::new().spawn(|| info!("i am thread!"));

    let f = finder::Finder::new(2);
    // f.scan("/Users/xuzhi/my/zip");
    f.scan("/Users/xuzhi/my/dev/morelike");

    f.join();

    let ten_millis = time::Duration::from_millis(1);
    thread::sleep(ten_millis);
    // {
    //     let pool = taskpool::ThreadPool::builder()
    //         .pool_size(4)
    //         .after_start(move |_size: usize| {
    //         })
    //         .before_stop(move |_size: usize| {
    //             // info!("{}", size);
    //         })
    //         .create();
    //     pool.spawn(|| {
    //         let ten_millis = time::Duration::from_millis(100);
    //         thread::sleep(ten_millis);
    //         info!("I am thread 0!");
    //         // finder::scan("/Users/xuzhi/my/zip");
    //     });
    //     pool.spawn(|| {
    //         let ten_millis = time::Duration::from_millis(100);
    //         thread::sleep(ten_millis);
    //         info!("I am thread 1!");
    //         // finder::scan("/Users/xuzhi/my/dev/morelike");
    //     });
    //     pool.spawn(|| {
    //         let ten_millis = time::Duration::from_millis(100);
    //         thread::sleep(ten_millis);
    //         info!("I am thread 2!");
    //     });
    // }

    debug!("end: ++++++++++++++++++++++");
}
