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

pub mod task;
// mod pool;
use task::{MsgPos, FinderMsg, Finder};

use std::thread;
use std::sync::mpsc::{channel,Sender,RecvError};
use std::sync::{Mutex, Arc};

fn main() {
    pretty_env_logger::init();
    info!("start");

    let p = "test";
    let p = "/Users/xuzhi/Music";

    let mut finder = Finder::new();

    finder.scan(p);

    loop {
        match finder.recv() {
            MsgPos::Start => {},
            MsgPos::End => { break; },
            MsgPos::ScanDir(path) => println!("{}", path),
            MsgPos::ScanFile(path) => println!("{}", path),
            MsgPos::EndScanThread(i) => {
                println!("thread end: {}", i);
                if i < 1 {
                    break;
                }
            },
            _ => {},
        }
    }


    // let mut task = Task::new();
    // let file_task = Arc::new(Mutex::new(task));

    // let flock = file_task.clone();
    // let handle = thread::spawn(move || {
    //     let p = "test";
    //     let mut task = flock.lock().unwrap();
    //     task.scan(p);
    // });
    // let p = "test";
    // task.scan(p);

    // loop {
    //     match task.recv_pos() {
    //         Ok(msg) => match msg {
    //             MsgPos::Start => println!("开始扫描: {}", ""),
    //             MsgPos::ScanDir(pos, desc) => println!("目录: {}={}", pos, desc),
    //             MsgPos::ScanFile(pos, desc) => println!("文件: {}={}", pos, desc),
    //             MsgPos::End => {
    //                 println!("{:?}", task);
    //                 println!("结束");
    //                 break;
    //             },
    //         },
    //         Err(RecvError) => panic!("no msg!"),
    //     }

    // }

    // handle.join().unwrap();
}
