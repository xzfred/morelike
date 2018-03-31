use std::fmt::{self, Formatter, Display};
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
use std::time::{Duration, SystemTime};
use std;


#[derive(Debug)]
pub struct FileInfo {
    name: String,
    path: String,
    size: u64,
    sum: Sha1,
    meta: Metadata,
}

// #[cfg(unix)]
// fn display_date(metadata: &Metadata, options: &getopts::Matches) -> String {
//     let secs = if options.opt_present("c") {
//         metadata.ctime()
//     } else {
//         metadata.mtime()
//     };
//     let time = time::at(Timespec::new(secs, 0));
//     strftime("%F %R", &time).unwrap()
// }

// #[cfg(not(unix))]
// #[allow(unused_variables)]
// fn display_date(metadata: &Metadata, options: &getopts::Matches) -> String {
//     if let Ok(mtime) = metadata.modified() {
//         let time = time::at(Timespec::new(
//             mtime
//                 .duration_since(std::time::UNIX_EPOCH)
//                 .unwrap()
//                 .as_secs() as i64,
//             0,
//         ));
//         strftime("%F %R", &time).unwrap()
//     } else {
//         "???".to_string()
//     }
// }

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
        write!(f, "name:{:<20} size:{:<15} accessed:{:14} modified:{:14}",
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
}

const BUFSIZE: usize = 1024;

pub type LikeList = HashMap<String, FileInfo>;

#[derive(Debug)]
pub struct FileTable {
    // buf: [u8; 1024],
    table: HashMap<u32, LikeList>,
}

impl Display for FileTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // let ft: &FileTable = *self;

        // try!(write!(f, "["));

        for (k, ref v) in self.table.iter() {
            try!(write!(f, "{index:<iw$} \n",
                        index=k, iw=12));
            for (ref name, ref fi) in v.iter() {
                try!(write!(f, "{nm:>width$} \n{sp:<width$} {file}\n", width=40, nm=name, file=fi, sp=""));
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
            // buf: [0; 1024],
        }
    }

    pub fn scan(&mut self, path: &str) {
        let path = Path::new(path);
        self.load(path, 0);
    }

    fn load(&mut self, parent: &Path, level: i32) {
        // let dirs = fs::read_dir(parent).unwrap();

        // let f_count = dirs.count();
        let dirs = fs::read_dir(parent).unwrap();

        for f in dirs {
            let ff = &f.unwrap().path(); 
            if ff.is_dir() {
                self.load(ff, level + 1);
            } else if ff.symlink_metadata().unwrap().file_type().is_symlink() {
                println!("is symlink: {}", ff.to_str().unwrap());
            } else if ff.is_file() {
                let file_info = FileInfo::new(ff);
                let sum = self.checksum(ff, &file_info);
                self.table.entry(sum)
                    .or_insert_with(HashMap::new)
                    .insert(file_info.path.clone(), file_info);
            }
        }
    }

    fn checksum(&self, file: &Path, file_info: &FileInfo) -> u32 {
        let display = file.display();
        let file: File = match File::open(&file) {
            Err(why) => panic!("couldn't open {}: {} {}",
                               display,
                               why.description(), file_info ),
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
