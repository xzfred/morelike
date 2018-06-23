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
use std::sync::{Arc, Mutex};

mod finder;
mod taskpool;
// use finder::*;

fn main() {
    pretty_env_logger::init();
    info!("start: ++++++++++++++++++++++");
    // finder::scan("/Users/xuzhi/my");

    // taskpool::ThreadPool::new().spawn(|| info!("i am thread!"));


    let wait = Arc::new(taskpool::WaitPool::new());
    let wait_after = wait.clone();
    let wait_before = wait.clone();
    {
        taskpool::ThreadPool::builder()
            .after_start(move |_size: usize| {
                wait_after.enter();
            })
            .before_stop(move |size: usize| {
                info!("{}", size);
                wait_before.leave();
            })
            .create().spawn(|| {
                let ten_millis = time::Duration::from_millis(100);
                thread::sleep(ten_millis);
                info!("I am thread!");
            });
    }

    wait.join();
    info!("end: ++++++++++++++++++++++");
}
