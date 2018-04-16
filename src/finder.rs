use std::fmt::{self, Display, Formatter};
use std::collections::HashMap;
use sha1::{Sha1, Digest};
use std::path::{Path, PathBuf};
use std::fs::{self, Metadata, File};
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

// 文件节点
#[derive(Debug)]
pub struct SearchFile {
    file: PathBuf,
    size: u64,
    crc: u32,
    sum: Sha1,
}

type Msg = Arc<SearchFile>;

pub enum Compare {
    Dir(Msg),
}

pub enum Lookup {
    Dir(Msg),
    Close,
}

pub enum Check {
    Sum(Msg),
    Crc(Msg),
    Close,
}

pub struct Finder {
    state: Rc<FinderState>,
}

struct Scan {
    state: Rc<RefCell<Post<Lookup, >>,
    idx: usize,
}

struct<P, C> Post {
    rx: Mutex<Receiver<P>>,
    tx: Mutex<Sender<P>>,
    tx_consumer: Mutex<Sender<C>>,
}

impl Future for Scan {
    
}
