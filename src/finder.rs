use std::fs::{self};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{thread, time};

use taskpool::*;

#[allow(dead_code)]
pub fn scan(path: &str) {
    let path = PathBuf::from(path);
    load(&path);
}

fn load(parent: &Path) {
    let dirs = fs::read_dir(parent).unwrap();

    for file in dirs {
        let ff = &file.unwrap().path();

        if ff.is_dir() {
            warn!("Dir: {}", ff.to_str().unwrap());
            load(ff);
        } else if ff.symlink_metadata().unwrap().file_type().is_symlink() {
            error!("Synlink: {}", ff.to_str().unwrap());
        } else if ff.is_file() {
            info!("File: {}", ff.to_str().unwrap());
        }
    }
}

pub enum FinderMsg {
    Dir(PathBuf, u32),
    File(PathBuf, u32),
    Close,
}

pub struct Scan {
    rx: Mutex<Receiver<FinderMsg>>,
    tx: Mutex<Sender<FinderMsg>>,
    cnt: AtomicUsize,
    sender: Arc<Fn(FinderMsg) -> bool + Send + Sync>,
    dir_count: AtomicUsize,
    file_count: AtomicUsize,
}

pub struct Finder {
    pool_size: usize,
    pool: ThreadPool,
    scan: Arc<Scan>,
}

impl Scan {
    pub fn new<F>(f: F) -> Scan
        where F: Fn(FinderMsg) -> bool + Send + Sync + 'static
    {
        let (sender, receiver) = channel::<FinderMsg>();
        Scan {
            rx: Mutex::new(receiver),
            tx: Mutex::new(sender),
            cnt: AtomicUsize::new(0),
            sender: Arc::new(f),
            dir_count: AtomicUsize::new(0),
            file_count: AtomicUsize::new(0),
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
        loop {
            let msg = self.rx.lock().unwrap().recv().unwrap();
            let sender = self.sender.clone();
            match msg {
                FinderMsg::Dir(path, level) => {
                    if sender(FinderMsg::Dir(path.clone(), level)) {
                        self.load(path, level);
                    }
                },
                FinderMsg::File(path, level) => {
                    sender(FinderMsg::File(path, level));
                },
                FinderMsg::Close => {
                    info!("Close");
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

    fn load(&self, parent: PathBuf, level: u32) {
        let dirs = fs::read_dir(parent).unwrap();

        for file in dirs {
            let ff = &file.unwrap().path();
            let buf = ff.to_owned();

            if ff.is_dir() {
                // warn!("Dir: {}", ff.to_str().unwrap());
                self.dir_count.fetch_add(1, Ordering::Relaxed);
                self.send(FinderMsg::Dir(buf, level + 1));
            } else if ff.symlink_metadata().unwrap().file_type().is_symlink() {
                // error!("Synlink: {}", ff.to_str().unwrap());
            } else if ff.is_file() {
                // warn!("File: {}", ff.to_str().unwrap());
                self.file_count.fetch_add(1, Ordering::Relaxed);
                self.send(FinderMsg::File(buf, level));
            }
        }
        self.cnt.fetch_sub(1, Ordering::Relaxed);
    }

}

impl Finder {
    pub fn new(size: usize) -> Finder {
        let pool = ThreadPool::builder().pool_size(size)
            .after_start(move |size: usize| {
                debug!("start: {}", size);
            }).before_stop(move |size: usize| {
                debug!("stop: {}", size);
            }).create();

        let scan = Scan::new(|msg: FinderMsg|{
            match msg {
                FinderMsg::Dir(_path, level) => {
                },
                FinderMsg::File(_path, level) => {
                },
                FinderMsg::Close => {},
            }
            true
        });

        let obj = Finder {
            pool_size: size,
            pool: pool,
            scan: Arc::new(scan),
        };

        for _i in 0..size {
            let s = obj.scan.clone();
            obj.pool.spawn(move || {
                s.run();
            });
        }

        return obj;
    }

    pub fn scan(&self, parent: &str) {
        self.scan.send(FinderMsg::Dir(PathBuf::from(parent), 0));
    }

    pub fn join(&self) {
        let ten_millis = time::Duration::from_millis(10);
        while self.scan.has_dir() {
            thread::sleep(ten_millis);
            debug!("wait scan dir! has:{}, dir:{}, file:{}",
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
