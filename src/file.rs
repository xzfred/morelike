
use std::collections::HashMap;
use sha1::Sha1;
use std::path::Path;
use std::fs;
use std::fs::FileType;

#[derive(Debug)]
pub struct FileInfo {
    file_type: FileType,
    name: String,
    path: String,
    size: u64,
    sum: Sha1,
}

impl FileInfo {
    pub fn new(path: &Path) -> FileInfo {
        FileInfo {
            file_type: path.metadata().unwrap().file_type(),
            name: String::from(path.file_name().unwrap().to_str().unwrap()),
            path: String::from(path.to_str().unwrap()),
            size: path.metadata().unwrap().len(),
            sum: Sha1::default(),
        }
    }
}

pub type LikeList = HashMap<String, FileInfo>;

pub struct FileTable(HashMap<u32, LikeList>);

impl FileTable {
    pub fn new() -> FileTable {
        FileTable(HashMap::new())
    }

    pub fn scan(&self, path: &str) {
        let path = Path::new(path);
        self.load(path, 0);
    }

    fn load(&self, parent: &Path, level: i32) {
        let dirs = fs::read_dir(parent).unwrap();
        let mut i: usize = 0;

        let f_count = dirs.count();
        let dirs = fs::read_dir(parent).unwrap();

        for f in dirs {
            i += 1;
            let ff = &f.unwrap().path(); // let buf = ff.to_str().unwrap().to_string();
            println!("{:?}", FileInfo::new(ff));
            if ff.is_dir() {
                self.load(ff, level + 1);
            }
        }
    }
}
