use std::fs::{self};
// use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{thread, time};
// use alloc::vec::Vec;

use taskpool::*;

// #[allow(dead_code)]
// pub fn scan(path: &str) {
//     let path = PathBuf::from(path);
//     load(&path);
// }

// fn load(parent: &Path) {
//     let dirs = fs::read_dir(parent).unwrap();

//     for file in dirs {
//         let ff = &file.unwrap().path();

//         if ff.is_dir() {
//             warn!("Dir: {}", ff.to_str().unwrap());
//             load(ff);
//         } else if ff.symlink_metadata().unwrap().file_type().is_symlink() {
//             error!("Synlink: {}", ff.to_str().unwrap());
//         } else if ff.is_file() {
//             info!("File: {}", ff.to_str().unwrap());
//         }
//     }
// }

pub enum FinderMsg {
    Dir(PathBuf, u32),
    File(PathBuf, u32),
    Close,
}

pub struct Scan {
    ignore: Mutex<Vec<String>>,
    rx: Mutex<Receiver<FinderMsg>>,
    tx: Mutex<Sender<FinderMsg>>,
    cnt: Arc<AtomicUsize>,
    sender: Arc<Fn(FinderMsg) -> bool + Send + Sync>,
    dir_count: Arc<AtomicUsize>,
    file_count: Arc<AtomicUsize>,
}

pub struct Finder {
    pool_size: usize,
    pool: ThreadPool,
    scan: Arc<Scan>,
}

impl Scan {
    fn is_ignore(ignore: &Vec<String>, path: &PathBuf) -> bool {
        let isit = path.file_name().unwrap().to_str().unwrap();
        for name in ignore {
            if name.eq(isit) {
                info!("{} vs {}", name, isit);
                return true;
            }
        }
        false
    }

    pub fn new<F>(f: F, ignore: Vec<String>) -> Scan
    where F: Fn(FinderMsg) -> bool + Send + Sync + 'static
    {
        let (sender, receiver) = channel::<FinderMsg>();
        Scan {
            ignore: Mutex::new(Vec::from(ignore)),
            rx: Mutex::new(receiver),
            tx: Mutex::new(sender),
            cnt: Arc::new(AtomicUsize::new(0)),
            sender: Arc::new(f),
            dir_count: Arc::new(AtomicUsize::new(0)),
            file_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn send(&self, msg: FinderMsg) {
        let msg = match msg {
            FinderMsg::Dir(path, level) => {
                self.cnt.fetch_add(1, Ordering::Relaxed);
                FinderMsg::Dir(path, level)
            },
            FinderMsg::File(path, level) => {
                FinderMsg::File(path, level)
            },
            FinderMsg::Close => FinderMsg::Close,
        };
        self.tx.lock().unwrap().send(msg).unwrap();
    }

    pub fn run(&self) {
        let ignore = self.ignore.lock().unwrap().clone();
        let sender = self.sender.clone();
        loop {
            let msg = self.rx.lock().unwrap().recv().unwrap();
            match msg {
                FinderMsg::Dir(path, level) => {
                    if sender(FinderMsg::Dir(path.clone(), level)) {
                        self.load(path, level, &ignore);
                    }
                },
                FinderMsg::File(path, level) => {
                    sender(FinderMsg::File(path, level));
                },
                FinderMsg::Close => {
                    // trace!("Close");
                    break;
                },
            }
        }
    }

    pub fn has_dir(&self) -> bool {
        let has = self.cnt.load(Ordering::Relaxed);
        // debug!("wait scan dir! has:{} dir:{}, file:{}",
        //        has,
        //        self.dir_count.load(Ordering::Relaxed),
        //        self.file_count.load(Ordering::Relaxed));
        has > 0
    }

    fn load(&self, parent: PathBuf, level: u32, ignore: &Vec<String>) {
        let dirs = fs::read_dir(parent).unwrap();

        // warn!("ID: {:?}", thread::current().id());
        for file in dirs {
            let ff = &file.unwrap().path();
            let buf = ff.to_owned();
            if Scan::is_ignore(&ignore, &buf) {
                continue;
            }

            if ff.is_dir() {
                // warn!("Dir: {}", ff.to_str().unwrap());
                self.dir_count.fetch_add(1, Ordering::Relaxed);
                self.send(FinderMsg::Dir(buf, level + 1));
            } else if ff.symlink_metadata().unwrap().file_type().is_symlink() {
                warn!("Synlink: {}", ff.to_str().unwrap());
            } else if ff.is_file() {
                // warn!("File: {}", ff.to_str().unwrap());
                self.file_count.fetch_add(1, Ordering::Relaxed);
                self.send(FinderMsg::File(buf, level));
            }
        }
        self.cnt.fetch_sub(1, Ordering::Relaxed);
    }

}

pub struct FinderBuilder {
    scan_pool_size: usize,
    ignore: Option<Vec<String>>,
    pool: Option<ThreadPool>,
}

impl FinderBuilder {
    pub fn new() -> FinderBuilder {
        FinderBuilder {
            scan_pool_size: 2,
            ignore: None,
            pool: None,
        }
    }

    pub fn ignore(&mut self, ignore: Vec<String>) -> &mut Self {
        self.ignore = Some(ignore);
        self
    }

    pub fn scan_pool_size(&mut self, size: usize) -> &mut Self {
        self.scan_pool_size = size;
        self
    }

    pub fn pool(&mut self, pool: ThreadPool) -> &mut Self {
        self.pool = Some(pool.clone());
        self
    }

    pub fn create<F>(&mut self, f: F) -> Finder
    where F: Fn(FinderMsg) -> bool + Send + Sync + 'static
    {
        let pool = match self.pool {
            Some(ref p) => p.clone(),
            None => ThreadPool::builder().pool_size(self.scan_pool_size).create(),
        };

        let ss = Scan::new(f, match self.ignore {
            Some(ref i) => i.clone(),
            None => vec![],});
        let obj = Finder {
            pool_size: self.scan_pool_size,
            pool: pool,
            scan: Arc::new(ss),
        };

        for _i in 0..self.scan_pool_size {
            let s = obj.scan.clone();
            obj.pool.spawn(move || {
                s.run();
            });
        }

        obj
    }
}

impl Finder {
    pub fn new<F>(f: F) -> Finder
    where F: Fn(FinderMsg) -> bool + Send + Sync + 'static
    {
        FinderBuilder::new().create(f)
    }

    pub fn scan(&self, parent: &str) {
        self.scan.send(FinderMsg::Dir(PathBuf::from(parent), 0));
    }

    pub fn join(&self) {
        let ten_millis = time::Duration::from_millis(10);
        while self.scan.has_dir() {
            thread::sleep(ten_millis);
            trace!("wait scan dir! has:{}, dir:{}, file:{}",
                   self.scan.cnt.load(Ordering::Relaxed),
                   self.scan.dir_count.load(Ordering::Relaxed),
                   self.scan.file_count.load(Ordering::Relaxed));
        }
    }
}

impl Drop for Finder {
    fn drop(&mut self) {
        for _i in 0..self.pool_size {
            self.scan.send(FinderMsg::Close);
        }
    }
}

