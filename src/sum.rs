use std::fs::{self, File};

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{thread, time};

use sha1::{Sha1, Digest};
// use crc32c_hw;

use finder::{Finder, FinderMsg};

// 文件节点
#[derive(Debug)]
pub struct SearchFile {
    file: PathBuf,
    size: u64,
    crc: u32,
    sum: Sha1,
}

impl SearchFile {
    pub fn new(f: PathBuf) -> SearchFile {
        let fs = f.metadata().unwrap().len();
        SearchFile {
            file: f,
            size: fs,
            crc: 0,
            sum: Sha1::default(),
        }
    }

    fn check_sha1(&mut self) -> std::io::Result<()> {
        let file: File = File::open(&self.file)?;

        let mut buf: [u8; BUFSIZE] = [0; BUFSIZE];

        let mut handle = file.take(BUFSIZE as u64);
        loop {
            let read_size = handle.read(&mut buf)?;
            if read_size > 0 {
                self.sum.input(buf.as_ref());
            }
            if read_size < BUFSIZE {
                break;
            }
        }
        Ok(())
    }

    fn checksum(&mut self) -> std::io::Result<()> {
        let file: File = File::open(&self.file)?;

        let mut buf: [u8; BUFSIZE] = [0; BUFSIZE];

        let mut handle = file.take(BUFSIZE as u64);
        let read_size = handle.read(&mut buf)?;
        if read_size > 0 {
            self.crc = crc32c_hw::compute(buf.as_ref());
        }
        Ok(())
    }
}

type FileList = Vec<Arc<SearchFile>>;
type FileGroupByCrc = HashMap<u32, FileList>;
type FileGroupBySize = HashMap<u64, FileGroupByCrc>;
type FileGroupBySum = HashMap<Sha1, FileList>;

struct Comparer {
    file_list: Mutex<FileGroupCrc>,
    file_dup: Mutex<FileGroup>,
}

impl Comparer {
    pub fn new() -> Comparer {
        Comparer {
        }
    }

    pub fn run(&self) {
        
    }

    fn compare(&mut self, fil: Arc<SearchFile>) {
        let mut file = fil.clone();
        let mut_file = Arc::get_mut(&mut file).unwrap();
        let group = self.file_group_crc.entry(mut_file.crc).or_insert(Vec::new());
        group.push(fil.clone());
        let count = group.len();
        if count > 1 {
            if count == 2 {
                let file = group[0].clone();
                // self.check_sha1(file);
            }
            // self.check_sha1(fil.clone());
        }
    }
}
