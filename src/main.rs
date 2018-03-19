use std::env;
use std::fs::{self, DirEntry, File, ReadDir};
use std::io::prelude::*;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    for param in args.iter() {
        println!("{}", param);
    }

    let dirs = fs::read_dir(Path::new(".")).unwrap();

    for f in dirs {
        println!("{}", f.unwrap().path().to_str().unwrap());
    }
}



















