use std::fs::{self, File, Metadata};
use std::path::Path;
use std::path::PathBuf;

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
