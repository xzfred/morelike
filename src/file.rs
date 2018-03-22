
use std::collections::HashMap;
use sha1::Sha1;
use std::path::Path;
use std::fs;
use std::fs::Metadata;
use std::fs::File;
use std::io::prelude::*;
use std::error::Error;
use crc32c_hw;

#[derive(Debug)]
pub struct FileInfo {
    name: String,
    path: String,
    size: u64,
    sum: Sha1,
    meta: Metadata,
}

impl FileInfo {
    pub fn new(path: &Path) -> FileInfo {
        let meta = path.metadata().unwrap();
        FileInfo {
            name: String::from(path.file_name().unwrap().to_str().unwrap()),
            path: String::from(path.to_str().unwrap()),
            size: meta.len(),
            sum: Sha1::default(),
            meta: meta,
        }
    }
}

const BUFSIZE: usize = 1024;

pub type LikeList = HashMap<String, FileInfo>;

pub struct FileTable {
    // buf: [u8; 1024],
    table: HashMap<u32, LikeList>,
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
        self.load(&path, 0);
        println!("{:?}", self.table);
    }

    fn load(&mut self, parent: &Path, level: i32) {
        // let dirs = fs::read_dir(parent).unwrap();

        // let f_count = dirs.count();
        let dirs = fs::read_dir(parent).unwrap();

        for f in dirs {
            let ff = &f.unwrap().path(); 
            if ff.is_dir() {
                self.load(ff, level + 1);
            } else {
                let file_info = FileInfo::new(ff);
                let sum = self.checksum(ff, &file_info);
                self.table.entry(sum)
                    .or_insert(HashMap::new())
                    .insert(file_info.path.clone(), file_info);
            }
        }
    }

    fn checksum(&self, file: &Path, file_info: &FileInfo) -> u32 {
        let display = file.display();
        let mut file: File = match File::open(&file) {
            Err(why) => panic!("couldn't open {}: {}",
                               display,
                               why.description() ),
            Ok(file) => file,
        };
        let mut buf: [u8; BUFSIZE] = [0; BUFSIZE];
        if file_info.size > BUFSIZE as u64 {
            file.read_exact(&mut buf).unwrap();
            crc32c_hw::compute(buf.as_ref())
        } else {
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).unwrap();
            crc32c_hw::compute(buffer.as_slice())
        }
    }
}
