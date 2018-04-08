use std::fmt::{self, Display, Formatter};
use std::collections::HashMap;
use sha1::Sha1;
use std::path::Path;
use std::path::PathBuf;
use std::fs;
use std::fs::Metadata;
use std::fs::File;
use std::io::prelude::*;
use std::error::Error;
use crc32c_hw;
use chrono::prelude::*;
use time;
use time::{strftime, Timespec};
use std::time::{Duration, Instant, SystemTime};
use std;
use indicatif::{HumanDuration, MultiProgress, ProgressBar, ProgressStyle};
use console::{style, Emoji};
use std::thread::{self,JoinHandle};
use std::sync::mpsc::{channel, Sender, Receiver, RecvError};
use std::sync::{Arc, Mutex};

// use pool::*;

// 位置消息
#[derive(Debug)]
pub enum MsgPos {
    Start,
    ScanDir(u32, String),
    ScanFile(u32, String),
    End,
}

// 文件节点
#[derive(Debug)]
pub struct SearchFile {
    file: Box<PathBuf>,
    size: u64,
    crc: u32,
    sum: Sha1,
}

#[derive(Debug)]
pub struct Task {

    // 文件队列
    file_list: Vec<Box<SearchFile>>,
}

// type FinderMsg = Box<PathBuf>;
pub enum FinderMsg {
    Dir(PathBuf),
    File(PathBuf),
}

#[derive(Debug)]
pub struct Finder {
    //计数器
    count_dir: u32,
    count_file: u32,

    pb_pos: Sender<FinderMsg>,
    pb_pos_recevier: Receiver<FinderMsg>,
    pool: Vec<ScanDir>,
    // handle: JoinHandle<_>,
}

struct ScanDir {
    tx: Sender<FinderMsg>,
    handle: thread::JoinHandle<()>,
}

impl ScanDir {
    pub fn new(tx: Sender<FinderMsg>) -> ScanDir {
        ScanDir {
            tx,
            handle: thread::spawn(),
        }
    }
}

impl Finder {
    pub fn new() -> Finder {
        let (sender, receiver) = channel::<FinderMsg>();
        Finder {
            count_dir: 0,
            count_file: 0,

            pb_pos: sender,
            pb_pos_recevier: receiver,
            pool: Vec::new(),
        }
    }

    pub fn recv(&self) -> Result<FinderMsg, RecvError> {
        self.pb_pos_recevier.recv()
    }

    pub fn scan(&mut self, path: &str) {
        let path = String::from(path);
        let local_self = Arc::new(self);
        let copy_self = local_self.clone();
        let handle = thread::spawn(move|| {
            let path = Path::new(&path);
            &copy_self.load(path, 0);
        });
        handle.join().unwrap();
    }

    fn load(&mut self, parent: &Path, level: i32) {
        let dirs = fs::read_dir(parent).unwrap();

        for f in dirs {
            let ff = &f.unwrap().path();
            if ff.is_dir() {
                self.count_dir += 1;
                self.pb_pos.send(FinderMsg::Dir(ff.to_path_buf())).unwrap();
                self.load(ff, level + 1);
            } else if ff.symlink_metadata().unwrap().file_type().is_symlink() {
                // println!("is symlink: {}", ff.to_str().unwrap());
            } else if ff.is_file() {
                self.count_file += 1;
                self.pb_pos.send(FinderMsg::File(ff.to_path_buf())).unwrap();
            }
        }
    }
}

impl SearchFile {
    pub fn new(f: &Path) -> SearchFile {
        SearchFile {
            file: Box::new(f.to_path_buf()),
            size: 0,
            crc: 0,
            sum: Sha1::default(),
        }
    }
}

impl Task {
    pub fn new() -> Task {
        Task {
            file_list: Vec::new(),
        }
    }
}
