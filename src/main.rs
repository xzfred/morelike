#![allow(unused_imports)]
#[macro_use] extern crate log;
extern crate pretty_env_logger;
extern crate clap;
extern crate indicatif;
extern crate console;
use clap::{Arg, App, ArgMatches};

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

extern crate actix;
extern crate futures;
extern crate tokio;

// pub mod task;
// use task::{MsgPos, FinderMsg, Finder};
use std::{thread, time};
// use std::sync::{Arc};

// use std::env;

mod finder;
// mod taskpool;
// mod sum;

fn main() {
    pretty_env_logger::init();
    debug!("start: ++++++++++++++++++++++");

    finder::run();
    finder::run();

    let ten_millis = time::Duration::from_millis(1);
    thread::sleep(ten_millis);
    debug!("end: ++++++++++++++++++++++");
}

// finder::scan("/Users/xuzhi/my");

// taskpool::ThreadPool::new().spawn(|| info!("i am thread!"));

// let comparer = sum::Comparer::new();
//     // f.scan("/Users/xuzhi/my/dev/morelike");
// comparer.run("/Users/fred/my/dev/morelike/test");
// comparer.run(&get_path("/my/zip"));
// comparer.run(&get_matchs());

// fn get_path(s: &str) -> String {
//     let path = match env::var_os("HOME") {
//         None => { println!("$HOME not set."); panic!(); }
//         Some(path) => path.to_str().unwrap().to_owned() + s,
//     };
//     path
// }

fn get_matchs() -> String {
    let matches = App::new("morelike")
        .version("0.1")
        .author("xzfred <xzfred@gmail.com>")
        .about("what is morelike?")
    // .arg(Arg::with_name("path")
    //      .short("c")
    //      .long("config")
    //      .value_name("FILE")
    //      .help("Sets a custom config file")
    //      .takes_value(true))
        .arg(Arg::with_name("INPUT")
             .help("Sets the scan to path")
             .required(true)
             .index(1))
    // .arg(Arg::with_name("v")
    //      .short("v")
    //      .multiple(true)
    //      .help("Sets the level of verbosity"))
        .get_matches();
    matches.value_of("INPUT").unwrap().to_owned()

    // if let Some(c) = matches.value_of("INPUT") {
    //     println!("Value for -c: {:?}", c);
    // }
}
