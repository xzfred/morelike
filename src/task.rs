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
    WaitDir,
    BeginScan,
    EndScan,
    EndScanThread(u32),
    ScanDir(String),
    ScanFile(String),
    End,
}

// 文件节点
#[derive(Debug)]
pub struct SearchFile {
    file: PathBuf,
    size: u64,
    crc: u32,
    sum: Sha1,
}

// type FinderMsg = Box<PathBuf>;
pub enum FinderMsg {
    Dir(PathBuf, u32),
    File(PathBuf),
    Close,
}

pub struct Finder {
    //计数器
    count_dir: u32,
    count_file: u32,
    count_wait: u32,
    count_scan: u32,
    scan_thread: u32,
    active_thread: u32,

    pb_pos: Sender<MsgPos>,
    pb_pos_recevier: Receiver<MsgPos>,
    finder_state: Arc<FinderState>,
}

struct FinderState {
    size: u32,
    rx: Mutex<Receiver<FinderMsg>>,
    tx: Mutex<Sender<FinderMsg>>,
    finder_tx: Mutex<Sender<MsgPos>>,
}

impl Drop for Finder {
    fn drop(&mut self) {
        for _ in 0..self.finder_state.size {
            self.finder_state.send(FinderMsg::Close);
        }
    }
}


impl FinderState {
    pub fn new(finder_tx: Sender<MsgPos>, size: u32) -> FinderState {
        let (sender, receiver) = channel::<FinderMsg>();
        FinderState {
            size,
            finder_tx: Mutex::new(finder_tx),
            rx: Mutex::new(receiver),
            tx: Mutex::new(sender),
        }
    }

    pub fn send(&self, msg: FinderMsg) {
        self.tx.lock().unwrap().send(msg).unwrap();
    }

    pub fn run(&self) {
        // println!("thread start");
        loop {
            self.finder_tx.lock().unwrap().send(MsgPos::WaitDir).unwrap();
            let msg = self.rx.lock().unwrap().recv().unwrap();
            self.finder_tx.lock().unwrap().send(MsgPos::BeginScan).unwrap();
            match msg {
                FinderMsg::Dir(path, level) => {
                    self.finder_tx.lock().unwrap().send(
                        MsgPos::ScanDir(String::from(path.to_str().unwrap()))).unwrap();
                    self.load(&path, level);
                },
                FinderMsg::File(path) => {
                    self.finder_tx.lock().unwrap().send(
                        MsgPos::ScanFile(String::from(path.to_str().unwrap()))).unwrap();
                },
                FinderMsg::Close => break,
            }

        }
        // println!("thread end");
        self.finder_tx.lock().unwrap().send(MsgPos::EndScanThread(0)).unwrap();
    }

    fn load(&self, parent: &Path, level: u32) {
        let dirs = fs::read_dir(parent).unwrap();

        for f in dirs {
            let ff = &f.unwrap().path();
            if ff.is_dir() {
                self.send(FinderMsg::Dir(ff.to_path_buf(), level + 1));
            } else if ff.symlink_metadata().unwrap().file_type().is_symlink() {
                // println!("is symlink: {}", ff.to_str().unwrap());
            } else if ff.is_file() {
                self.send(FinderMsg::File(ff.to_path_buf()));
            }
        }
    }
}

impl Finder {
    pub fn new() -> Finder {
        let (sender, receiver) = channel::<MsgPos>();
        let size = 2;
        let state = Arc::new(FinderState::new(sender.clone(), size));

        for _ in 0..size {
            let mut pool = state.clone();
            thread::spawn(move || pool.run());
        }

        Finder {
            scan_thread: size,
            active_thread: size,
            count_dir: 0,
            count_file: 0,
            count_wait: 0,
            count_scan: 0,

            pb_pos: sender.clone(),
            pb_pos_recevier: receiver,
            finder_state: state,
        }
    }

    pub fn recv(&mut self) -> MsgPos {
        match self.pb_pos_recevier.recv() {
            Ok(msg) => match msg {
                MsgPos::EndScanThread(_i) => {
                    self.active_thread -= 1;
                    return MsgPos::EndScanThread(self.active_thread);
                },
                MsgPos::WaitDir => {
                    self.count_wait += 1;
                    if self.count_wait == self.scan_thread && self.count_scan > 0 {
                        self.pb_pos.send(MsgPos::EndScan).unwrap();
                        for _i in 0..self.scan_thread {
                            self.finder_state.send(FinderMsg::Close);
                        }
                    }
                    return MsgPos::WaitDir;
                },
                MsgPos::BeginScan => {
                    self.count_wait -= 1;
                    self.count_scan += 1;
                    return MsgPos::BeginScan;
                },
                MsgPos::ScanDir(path) => {
                    self.count_dir += 1;
                    return MsgPos::ScanDir(path);
                },
                MsgPos::ScanFile(path) => {
                    self.count_file += 1;
                    return MsgPos::ScanFile(path);
                },
                _ => return msg,
            },
            Err(RecvError) => panic!("recv: {}", RecvError),
        };
    }

    pub fn scan(&mut self, path: &str) {
        self.pb_pos.send(MsgPos::Start).unwrap();
        self.finder_state.send(FinderMsg::Dir(PathBuf::from(path), 0));
    }
}
