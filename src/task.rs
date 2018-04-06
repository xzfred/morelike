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
use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver, RecvError};

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

    
    pb_pos: Sender<MsgPos>,
    pb_pos_recevier: Receiver<MsgPos>,
}

#[derive(Debug)]
struct Finder {
    //计数器
    count_dir: u32,
    count_file: u32,
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
        let (sender, receiver) = channel::<MsgPos>();
        Task {
            pb_pos: sender,
            pb_pos_recevier: receiver,

            count_dir: 0,
            count_file: 0,
            // buf: [0; 1024],

            file_list: Vec::new(),
        }
    }

    pub fn recv_pos(&self) -> Result<MsgPos, RecvError> {
        self.pb_pos_recevier.recv()
    }

    pub fn scan(&mut self, path: &str) {
        let path = Path::new(path);
        self.pos(MsgPos::Start);
        self.load(path, 0);
        self.pos(MsgPos::End);
    }

    fn pos(&self, pos: MsgPos) {
        self.pb_pos.send(pos).unwrap();
    }

    fn add_dir(&mut self, dir: &PathBuf) {
        self.count_dir += 1;
        self.pos(MsgPos::ScanDir(self.count_dir, String::from(dir.to_str().unwrap())));
    }

    fn add_file(&mut self, file: &PathBuf) {
        self.file_list.push(Box::new(SearchFile::new(file)));
        self.count_file += 1;
        self.pos(MsgPos::ScanFile(self.count_file, String::from(file.to_str().unwrap())));
    }

    fn load(&mut self, parent: &Path, level: i32) {
        let dirs = fs::read_dir(parent).unwrap();

        for f in dirs {
            let ff = &f.unwrap().path();
            if ff.is_dir() {
                self.add_dir(ff);
                self.load(ff, level + 1);
            } else if ff.symlink_metadata().unwrap().file_type().is_symlink() {
                // println!("is symlink: {}", ff.to_str().unwrap());
            } else if ff.is_file() {
                self.add_file(ff);
            }
        }
    }
}
