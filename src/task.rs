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

// 文件节点
#[derive(Debug)]
pub struct SearchFile {
    file: Box<PathBuf>,
}

#[derive(Debug)]
pub struct Task {
    //计数器
    count_dir: u32,
    count_file: u32,

    // 文件队列
    file_list: Vec<Box<SearchFile>>,
}

impl SearchFile {
    pub fn new(f: &Path) -> SearchFile {
        SearchFile {
            file: Box::new(f.to_path_buf()),
        }
    }
}

impl Task {
    pub fn new() -> Task {
        Task {
            count_dir: 0,
            count_file: 0,
            // buf: [0; 1024],

            file_list: Vec::new(),
        }
    }

    pub fn scan(&mut self, path: &str) {
        let path = Path::new(path);
        self.load(path, 0);
    }

    fn add_dir(&mut self, _dir: &PathBuf) {
        self.count_dir += 1;
    }

    fn add_file(&mut self, file: &PathBuf) {
        self.file_list.push(Box::new(SearchFile::new(file)));
        self.count_file += 1;
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
