use std::env;
use std::fs::{self, DirEntry, File, ReadDir};
use std::io::prelude::*;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    for param in args.iter() {
        println!("{}", param);
    }

    let root = Path::new(".");
    load(&root, 0);
}

// fn load<P: AsRef<Path>>(parent: P) {
fn load(parent: &Path, level: i32) {
    let dirs = fs::read_dir(parent).unwrap();
    let mut i: usize = 0;

    let has_flag = "├";
    let last_flag = "└";
    let f_count = dirs.count();
    let dirs = fs::read_dir(parent).unwrap();

    for f in dirs {
        i += 1;
        let flag = if i == f_count { last_flag } else { has_flag };
        let ff = &f.unwrap().path(); // let buf = ff.to_str().unwrap().to_string();
        for _l in 0..level {
            print!("{}", "  ");
        }
        match ff.file_name() {
            None => println!("{}", ""),
            Some(b) => println!("{}─{}", flag, b.to_str().unwrap()),
        }
        //            '├─';
        // println!("|-{}", &buf[2..]);
        if ff.is_dir() {
            load(ff, level + 1);
        }
    }
}
