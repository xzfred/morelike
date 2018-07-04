
use std::io::{Read, Result};
use std::fs::{File};
use std::rc::Rc;
use std::collections::HashMap;
use std::hash::{Hasher, Hash};
use std::path::{// Path ,
    PathBuf};
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
// use std::sync::mpsc::{channel, Sender, Receiver};
// use std::sync::atomic::{AtomicUsize, Ordering};
// use std::{thread, time};

// use sha1::{Sha1, Digest};
// use crc32c_hw;

// use generic_array::{GenericArray};
// use digest::generic_array::typenum::{U20};

use finder::{Finder, FinderMsg};

use twox_hash::{XxHash};


const BUFSIZE: usize = 1024;

// type Output<N> = GenericArray<u8, N>;
// static DEFAULT_SUM: [u8; 20] = [0; 20];

type HashSum = u64;

pub fn checksum(path: &PathBuf, all: bool) -> Result<HashSum> {
    let file: File = File::open(path)?;

    let mut buf: [u8; BUFSIZE] = [0; BUFSIZE];

    let mut handle = file.take(BUFSIZE as u64);
    let mut hash = XxHash::with_seed(0);
    loop {
        let read_size = handle.read(&mut buf)?;
        if read_size > 0 {
            hash.write(&buf.as_ref());
            if !all {
                break;
            }
        } else {
            break;
        }
    }
    let sum = hash.finish();
    Ok(sum)
}

// 文件节点
#[derive(Debug)]
pub struct SearchFile {
    file: PathBuf,
    size: u64,
    crc: Option<HashSum>,
    sum: Option<HashSum>,
}

impl SearchFile {
    pub fn new(f: PathBuf) -> SearchFile {
        let fs = f.metadata().unwrap().len();
        SearchFile {
            file: f,
            size: fs,
            crc: None,
            sum: None,
        }
    }

    pub fn next(&self) -> i32 {
        match self.crc {
            None => 0,
            Some(_) => {
                match self.sum {
                    None => 1,
                    Some(_) => 2,
                }
            }
        }
    }

    pub fn checkcrc(&mut self) -> Result<HashSum> {
        match self.crc {
            Some(sum) => return Ok(sum),
            None => {
                let sum = checksum(&self.file, false)?;
                self.crc = Some(sum);
                Ok(sum)
            }
        }
    }

    pub fn checksum(&mut self) -> Result<HashSum> {
        match self.sum {
            Some(sum) => return Ok(sum),
            None => {
                let sum = checksum(&self.file, false)?;
                self.sum = Some(sum);
                Ok(sum)
            }
        }
    }

    // pub fn check_sha1(&mut self) -> Result<()> {
    //     let mut file: File = File::open(&self.file)?;

    //     let mut buf: [u8; BUFSIZE] = [0; BUFSIZE];

    //     // let mut handle = file.take(BUFSIZE as u64);
    //     let mut sha1 = Sha1::default();
    //     loop {
    //         let read_size = file.read(&mut buf)?;
    //         if read_size > 0 {
    //             sha1.input(buf.as_ref());
    //         }
    //         if read_size < BUFSIZE {
    //             break;
    //         }
    //     }
    //     self.sum = sha1.result();
    //     Ok(())
    // }
}

// type Sha1Result = [u8; 20];
type TFile = Arc<Mutex<SearchFile>>;
type TFileList = Vec<TFile>;
// type TFileGroupBySize = HashMap<u64, Option<TFileList>>;
// type TFileGroupByCrc = HashMap<HashSum, Option<TFileList>>;
// type TFileGroupBySum = HashMap<HashSum, Option<TFileList>>;

struct FileTable<K>(Mutex<HashMap<K, Option<TFileList>>>, i32);

impl<K> FileTable<K>
where K: Eq + Hash
{
    pub fn new(step: i32) -> FileTable<K> {
        let map: HashMap<K, Option<TFileList>> = HashMap::new();
        FileTable(Mutex::new(map), step)
    }

    pub fn entry(&mut self, k: K, f: TFile) -> Option<Option<TFile>> {
        let mut lock = self.0.lock().unwrap();
        let node = lock.entry(k).or_insert(Some(TFileList::new()));
        let next = match node {
            Some(list) => {
                if list.len() < 1 {
                    list.push(f);
                    None
                } else if self.1 == 2 {
                    if let None = list.iter()
                        .find(|&&x| {
                            let x = x.clone();
                            x.lock().unwrap().file == f.lock().unwrap().file  
                        }) {
                        list.push(f);
                    }
                    None
                } else {
                    Some(Some(list.pop().unwrap()))
                }
            },
            None => {
                Some(None)
            }
        };
        match next {
            Some(ofile) => match ofile {
                Some(file) => {
                    lock.remove(&k);
                    lock.insert(k, None);
                },
                None => {}
            },
            None => {}
        }
        next
    }
}

pub struct ComparerState {
    list_by_size: FileTable<u64>,
    list_by_crc: FileTable<HashSum>,
    list_by_sum: FileTable<HashSum>,
}

pub struct Comparer {
    finder: Finder,
    state: Arc<ComparerState>,
}

impl ComparerState {
    pub fn new() -> ComparerState {
        ComparerState {
            list_by_size: FileTable::new(0),
            list_by_crc: FileTable::new(1),
            list_by_sum: FileTable::new(2),
        }
    }

    pub fn compare(&mut self, fil: PathBuf) {
        let file = SearchFile::new(fil);
        let file = Arc::new(Mutex::new(file));
        self.next(0, file);
    }

    fn next(&mut self, step: i32, f: TFile) {
        let queue = match step {
            0 => {
                let file_size = f.lock().unwrap().size;
                self.list_by_size.entry(file_size, f.clone())
            },
            1 => {
                let file_size = f.lock().unwrap().checkcrc().unwrap();
                self.list_by_crc.entry(file_size, f.clone())
            },
            2 => {
                let file_size = f.lock().unwrap().checksum().unwrap();
                self.list_by_sum.entry(file_size, f.clone())
            },
            _ => {None}
        };
        match queue {
            Some(ofile) => match ofile {
                Some(old_file) => {
                    self.next(step + 1, old_file);
                    self.next(step + 1, f);
                },
                None => {},
            },
            None => {},
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
        // println!("{:?}", self.state.file_list);
        // println!("{:?}", self.state.file_dup);
    }
}
