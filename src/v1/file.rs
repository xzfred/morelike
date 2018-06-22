#[warn(dead_code)]
use std::fmt::{self, Display, Formatter};
use std::collections::HashMap;
use sha1::Sha1;
use std::path::Path;
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

#[derive(Debug)]
pub struct FileInfo {
    name: String,
    path: String,
    size: u64,
    sum: Sha1,
    meta: Metadata,
}

fn display_date(date: &SystemTime) -> String {
    let ttime = time::at(Timespec::new(
        date.duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
        0,
    ));
    strftime("%F %R", &ttime).unwrap()
}

impl Display for FileInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "name:{:<20} size:{:<15} accessed:{:14} modified:{:14}",
            self.name,
            self.size,
            display_date(&self.meta.accessed().unwrap()),
            display_date(&self.meta.modified().unwrap())
        )
    }
}

impl FileInfo {
    pub fn new(path: &Path) -> FileInfo {
        let meta = path.metadata().unwrap();
        FileInfo {
            name: String::from(path.file_name().unwrap().to_str().unwrap()),
            path: String::from(path.to_str().unwrap()),
            size: meta.len(),
            sum: Sha1::default(),
            meta,
        }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }
}

const BUFSIZE: usize = 1024;

static LOOKING_GLASS: Emoji = Emoji("🔍  ", "");

pub type LikeList = HashMap<String, FileInfo>;

// #[derive(Debug)]
pub struct FileTable {
    // buf: [u8; 1024],
    table: HashMap<u32, LikeList>,
    pb: ProgressBar,
    count_dir: u32,
    count_file: u32,
}

impl Display for FileTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // let ft: &FileTable = *self;

        // try!(write!(f, "["));

        for (k, ref v) in self.table.iter() {
            try!(write!(f, "{index:<iw$} \n", index = k, iw = 12));
            for (ref name, ref fi) in v.iter() {
                try!(write!(
                    f,
                    "{nm:>width$} \n{sp:<width$} {file}\n",
                    width = 40,
                    nm = name,
                    file = fi,
                    sp = ""
                ));
            }
        }

        // 加上配对中括号，并返回一个 fmt::Result 值
        write!(f, "]")
    }
}

impl FileTable {
    pub fn new() -> FileTable {
        FileTable {
            table: HashMap::new(),
            pb: ProgressBar::new(100),
            count_dir: 0,
            count_file: 0,
            // buf: [0; 1024],
        }
    }

    pub fn scan(&mut self, path: &str) {
        let spinner_style = ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{prefix:.bold.dim} {spinner} {wide_msg}");

        println!(
            "{} {}Resolving packages...",
            style("[1/4]").bold().dim(),
            LOOKING_GLASS
        );
        // println!("{} {}Fetching packages...", style("[2/4]").bold().dim(), TRUCK);
        // println!("{} {}Linking dependencies...", style("[3/4]").bold().dim(), CLIP);
        self.pb.set_style(spinner_style.clone());
        self.pb.set_prefix(&format!("[{}/?]", 1));

        let path = Path::new(path);
        self.load(path, 0);
    }

    pub fn exact(&self) {
        for (key, val) in &self.table {
            if val.len() > 1 {
                for (path, _file) in val {
                    println!("{}:, {:?}", key, path);
                }
            }
        }
    }

    fn load(&mut self, parent: &Path, level: i32) {
        // let dirs = fs::read_dir(parent).unwrap();

        // let f_count = dirs.count();
        let dirs = fs::read_dir(parent).unwrap();

        for f in dirs {
            let ff = &f.unwrap().path();
            if ff.is_dir() {
                self.count_dir += 1;
                self.load(ff, level + 1);
            } else if ff.symlink_metadata().unwrap().file_type().is_symlink() {
                // println!("is symlink: {}", ff.to_str().unwrap());
            } else if ff.is_file() {
                self.count_file += 1;
                let file_info = FileInfo::new(ff);

                let tname = file_info
                    .name
                    .chars()
                    .into_iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>();
                self.pb.set_message(&format!(
                    "dirs:{},files:{}, {} >> {}",
                    self.count_dir,
                    self.count_file,
                    ff.parent().unwrap().to_str().unwrap(),
                    if tname.len() > 10 {
                        tname[0..10].join("")
                    } else {
                        tname[0..].join("")
                    }
                ));
                self.pb.inc(1);
                // thread::sleep(Duration::from_millis(10));

                let sum = self.checksum(ff, &file_info);
                self.table
                    .entry(sum)
                    .or_insert_with(HashMap::new)
                    .insert(file_info.path.clone(), file_info);
            }
        }
    }

    fn checksum(&self, file: &Path, file_info: &FileInfo) -> u32 {
        let display = file.display();
        let file: File = match File::open(&file) {
            Err(why) => panic!(
                "couldn't open {}: {} {}",
                display,
                why.description(),
                file_info
            ),
            Ok(file) => file,
        };
        let mut buf: [u8; BUFSIZE] = [0; BUFSIZE];

        let mut handle = file.take(BUFSIZE as u64);
        handle.read(&mut buf).unwrap();
        crc32c_hw::compute(buf.as_ref())

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
