
use std::io::{Read, Result};
use std::fs::{File};
use std::collections::HashMap;
use std::path::{// Path ,
    PathBuf};
use std::sync::{Arc, Mutex};
// use std::sync::mpsc::{channel, Sender, Receiver};
// use std::sync::atomic::{AtomicUsize, Ordering};
// use std::{thread, time};

use sha1::{Sha1, Digest};
use crc32c_hw;

use generic_array::{GenericArray};
use digest::generic_array::typenum::{U20};

use finder::{Finder, FinderMsg};

const BUFSIZE: usize = 1024;

type Output<N> = GenericArray<u8, N>;

static DEFAULT_SUM: [u8; 20] = [0; 20];

// 文件节点
#[derive(Debug)]
pub struct SearchFile {
    file: PathBuf,
    size: u64,
    crc: u32,
    sum: Output<U20>,
}

impl SearchFile {
    pub fn new(f: PathBuf) -> SearchFile {
        let fs = f.metadata().unwrap().len();
        SearchFile {
            file: f,
            size: fs,
            crc: 0,
            sum: arr![u8; 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        }
    }

    pub fn check_sha1(&mut self) -> Result<()> {
        let mut file: File = File::open(&self.file)?;

        let mut buf: [u8; BUFSIZE] = [0; BUFSIZE];

        // let mut handle = file.take(BUFSIZE as u64);
        let mut sha1 = Sha1::default();
        loop {
            let read_size = file.read(&mut buf)?;
            if read_size > 0 {
                sha1.input(buf.as_ref());
            }
            if read_size < BUFSIZE {
                break;
            }
        }
        self.sum = sha1.result();
        Ok(())
    }

    pub fn is_default_sha1(&self) -> bool {
        self.sum[..] == DEFAULT_SUM
    }

    pub fn checksum(&mut self) -> Result<()> {
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

// type Sha1Result = [u8; 20];
type FileList = Vec<SearchFile>;
type FileGroupByCrc = HashMap<u32, FileList>;
type FileGroupBySize = HashMap<u64, FileGroupByCrc>;
type FileGroupBySum = HashMap<Output<U20>, FileList>;

pub struct ComparerState {
    file_list: Mutex<FileGroupBySize>,
    file_dup: Mutex<FileGroupBySum>,
}

pub struct Comparer {
    finder: Finder,
    state: Arc<ComparerState>,
}

impl ComparerState {
    pub fn new() -> ComparerState {
        ComparerState {
            file_list: Mutex::new(FileGroupBySize::new()),
            file_dup: Mutex::new(FileGroupBySum::new()),
        }
    }

    pub fn compare(&self, fil: PathBuf) {
        let afile = {
            let mut file = SearchFile::new(fil);
            file.checksum().unwrap();
            // Arc::new(file)
            file
        };
        let mut lock = self.file_list.lock().unwrap();
        let list = lock.entry(afile.size)
            .or_insert(FileGroupByCrc::new()).entry(afile.crc)
            .or_insert(FileList::new());
        list.push(afile);

        // info!("afile: {:?}", afile);
        if list.len() > 0 {
            let mut dup_list = self.file_dup.lock().unwrap();
            for item in list {
                if item.is_default_sha1() {
                    item.check_sha1().unwrap();
                        dup_list.entry(item.sum).or_insert(FileList::new())
                        .push(*item);
                }
                // let mut mut_item = item.clone();
                // let mut_file = .unwrap();
                // match Arc::get_mut(&mut mut_item) {
                //     Some(mut_file) => {
                //         info!("mut: {:?}", mut_file);
                //         if mut_file.is_default_sha1() {
                //             mut_file.check_sha1().unwrap();
                //             self.file_dup.lock().unwrap()
                //                 .entry(mut_file.sum)
                //                 .or_insert(FileList::new());
                //         }
                //     },
                //     None => {
                //         info!("None: {:?}", item);
                //     },
                // }
            }
        }
    }
}

impl Comparer {
    pub fn new() -> Comparer {
        let s = Arc::new(ComparerState::new());
        let ss = s.clone();
        Comparer {
            finder: Finder::new(1, vec![
                ".git".to_owned(),
                "target".to_owned(),
            ], move |msg: FinderMsg| {
                match msg {
                    FinderMsg::Dir(_path, _level) => {
                    },
                    FinderMsg::File(path, _level) => {
                        // let mut ssm = ss.clone();
                        // let ssss = Arc::get_mut(&mut ssm).unwrap();
                        // ssss.compare(path);
                        info!("{:?}", path);
                        ss.compare(path);
                    },
                    _ => {},
                }
                true
            }),
            state: s,
        }
    }

    pub fn run(&self, parent: &str) {
        self.finder.scan(parent);
        self.finder.join();
        println!("{:?}", self.state.file_list);
        println!("{:?}", self.state.file_dup);
    }
}
