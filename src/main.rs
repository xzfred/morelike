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

// mod file;
// use file::FileTable;
// use std::convert::AsRef;

pub mod task;
use task::{Task,MsgPos};

use std::thread;
use std::sync::mpsc::{channel,Sender,RecvError};
use std::sync::{Mutex, Arc};

fn main() {
    pretty_env_logger::init();
    info!("start");
    // let p = "/Users/xuzhi/Music";

    let (sender, receiver) = channel::<MsgPos>();

    let task = Task::new(sender);
    let file_task = Arc::new(Mutex::new(task));

    let flock = file_task.clone();
    let handle = thread::spawn(move || {
        let p = "test";
        let mut task = flock.lock().unwrap();
        task.scan(p);
    });

    loop {
        match receiver.recv() {
            Ok(msg) => match msg {
                MsgPos::Start => println!("开始扫描: {}", ""),
                MsgPos::ScanDir(pos, desc) => println!("目录: {}={}", pos, desc),
                MsgPos::ScanFile(pos, desc) => println!("文件: {}={}", pos, desc),
                MsgPos::End => {
                    println!("{:?}", *file_task.lock().unwrap());
                    println!("结束");
                    break;
                },
            },
            Err(RecvError) => panic!("no msg!"),
        }

    }

    handle.join().unwrap();
    // let mut table = FileTable::new();
    // table.scan(p);
    // //table.scan("test");
    // println!("{}", table);

    // // println!("{:?}", table);
    // table.exact();
}
