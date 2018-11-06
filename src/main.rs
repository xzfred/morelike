#[allow(unused_imports)]

use std::path::{Path, PathBuf};
use std::io;
use std::fs::{self, DirEntry};

#[macro_use] extern crate log;
extern crate pretty_env_logger;
extern crate tokio_fs;
extern crate tokio;
extern crate futures;

use futures::{Future, Stream};
// use tokio_fs::{ReadDir};
use std::{thread, time};


fn main() {
    pretty_env_logger::init();
    debug!("start: ++++++++++++++++++++++");

    debug!("end: ++++++++++++++++++++++");

    load(".");
}

fn load<P>(path: P)
where
    P: AsRef<Path> + Send + 'static,
{
    let fut = tokio_fs::read_dir(path).flatten_stream().for_each(|dir| {
        if dir.path().is_dir() {
            println!("dir: {:?}", dir.path());
            load(dir.path());
        } else {
            println!("file: {:?}", dir.path());
        }
        let ten_millis = time::Duration::from_millis(10);
        thread::sleep(ten_millis);
        Ok(())
    }).map_err(|err| {
        eprintln!("Error: {:?}", err);
        ()
    });
    tokio::run(fut);
}
