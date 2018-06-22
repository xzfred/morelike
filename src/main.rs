#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![allow(unused_imports)]
extern crate sha1;
#[macro_use]
extern crate log;
extern crate pretty_env_logger;

extern crate adler32;
extern crate chrono;
extern crate console;
// extern crate crc32c_hw;
extern crate indicatif;
extern crate time;

// pub mod task;
// use task::{MsgPos, FinderMsg, Finder};

mod finder;
mod taskpool;
// use finder::*;

use std::thread;
use std::sync::mpsc::{channel,Sender,RecvError};
use std::sync::{Mutex, Arc};

fn main() {
    pretty_env_logger::init();
    info!("start: ++++++++++++++++++++++");
    finder::scan("/Users/xuzhi/my");
    info!("end: ++++++++++++++++++++++");
}
