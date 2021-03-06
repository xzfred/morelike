use std::fmt::{self, Display, Formatter};
use std::collections::HashMap;
use sha1::{Sha1, Digest};
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
    SumFile(Arc<SearchFile>),
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
    File(Arc<SearchFile>),
    Close,
}

pub enum FileCheckMsg {
    Sum(Arc<SearchFile>),
    Crc(Arc<SearchFile>),
    Close,
}

const BUFSIZE: usize = 1024;

type FileGroup = HashMap<u64, Vec<Arc<SearchFile>>>;
type FileGroupCrc = HashMap<u32, Vec<Arc<SearchFile>>>;

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

    file_list: Mutex<Vec<Arc<SearchFile>>>,
    file_group: FileGroup,
    file_group_crc: FileGroupCrc,

    rx_work: Arc<Mutex<Receiver<FileCheckMsg>>>,
    tx_work: Mutex<Sender<FileCheckMsg>>,
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
            // file_group: Mutex::new(HashMap::new()),
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
                        MsgPos::SumFile(path)).unwrap();
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
                let meta = ff.metadata().unwrap();
                let file = SearchFile {
                    file: ff.to_path_buf(),
                    size: meta.len(),
                    crc: 0,
                    sum: Sha1::default(),
                };

                self.send(FinderMsg::File(Arc::new(file)));
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

        let (tx, rx) = channel::<FileCheckMsg>();
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

            file_list: Mutex::new(Vec::new()),
            file_group: HashMap::new(),
            file_group_crc: HashMap::new(),

            rx_work: Arc::new(Mutex::new(rx)),
            tx_work: Mutex::new(tx),
        }
    }

    pub fn recv(&mut self) -> MsgPos {
        match self.pb_pos_recevier.recv() {
            Ok(msg) => match msg {
                MsgPos::EndScanThread(i) => {
                    info!("{:?}", i);
                    self.active_thread -= 1;
                    if self.active_thread < 1 && i < 1 {
                        self.tx_work.lock().unwrap().send(FileCheckMsg::Close).unwrap();
                        self.active_thread += 1;
                    }
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
                MsgPos::SumFile(file) => {
                    self.count_file += 1;
                    // println!("{:?}", file);
                    debug!("{:?}", file);
                    self.add_file(file.clone());
                    return MsgPos::ScanFile(String::from(file.file.to_str().unwrap()));
                },
                MsgPos::ScanFile(path) => {
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
        self.run();
    }

    fn add_file(&mut self, file: Arc<SearchFile>) {
        self.file_list.lock().unwrap().push(file.clone());
        // let mut map = self.file_group.lock().unwrap();
        let group = self.file_group.entry(file.size).or_insert(Vec::new());
        group.push(file.clone());
        let count = group.len();
        if count > 1 {
            self.tx_work.lock().unwrap().send(FileCheckMsg::Crc(file.clone())).unwrap();
            if count == 2 {
                self.tx_work.lock().unwrap().send(FileCheckMsg::Crc(group[0].clone())).unwrap();
            }
        }
    }

    pub fn run(&self) {
        let rx = self.rx_work.clone();
        // let tx = self.tx_work.lock().unwrap().clone();
        let tx = self.pb_pos.clone();
        thread::spawn(move || {
            info!("start work!");
            loop {
                match rx.lock().unwrap().recv() {
                    Ok(msg) => match msg {
                        FileCheckMsg::Crc(file) => {
                            info!("dump {:?}", file);
                            Finder::checksum(file);
                        },
                        FileCheckMsg::Sum(file) => {
                            info!("dump {:?}", file);
                            Finder::check_sha1(file);

                        },
                        FileCheckMsg::Close => {
                            info!("exit work!");
                            break;
                        }
                    },
                    Err(RecvError) => panic!("recv: {}", RecvError),
                }
            }
            tx.send(MsgPos::EndScanThread(1)).unwrap();
        });
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

    fn check_sha1(file: Arc<SearchFile>) {
        let mut file = file.clone();
        let node = Arc::get_mut(&mut file).unwrap();
        let display = node.file.display();
        let file: File = match File::open(&node.file) {
            Err(why) => panic!(
                "couldn't open {}: {} ",
                display,
                why.description()
            ),
            Ok(file) => file,
        };

        let mut buf: [u8; BUFSIZE] = [0; BUFSIZE];

        let mut handle = file.take(BUFSIZE as u64);
        loop {
            handle.read(&mut buf).unwrap();
            node.sum.input(buf.as_ref());
        }
        // crc32c_hw::compute(buf.as_ref())
    }

    fn checksum(file: Arc<SearchFile>) {
        let mut file = file.clone();
        let node = Arc::get_mut(&mut file).unwrap();

        let display = node.file.display();
        let file: File = match File::open(&node.file) {
            Err(why) => panic!(
                "couldn't open {}: {} ",
                display,
                why.description()
            ),
            Ok(file) => file,
        };

        let mut buf: [u8; BUFSIZE] = [0; BUFSIZE];

        let mut handle = file.take(BUFSIZE as u64);
        handle.read(&mut buf).unwrap();
        node.crc = crc32c_hw::compute(buf.as_ref());

        // if file_info.size > BUFSIZE as u64 {
        //     file.read_exact(&mut buf).unwrap();
        //     crc32c_hw::compute(buf.as_ref())
        // } else {
        //     let mut buffer = Vec::new();
        //     file.read_to_end(&mut buffer).unwrap();
        //     crc32c_hw::compute(buffer.as_slice())
        // }
    }
}
