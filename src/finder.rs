use std::fs::{self};
use std::io;
use std::path::{Path, PathBuf};

use actix::prelude::*;
use futures::Future;
use tokio;

use std::{thread, time};


pub struct Scan {
    ignore: Vec<String>,
}

impl Scan {
    fn is_ignore(ignore: &Vec<String>, path: &PathBuf) -> bool {
        let isit = path.file_name().unwrap().to_str().unwrap();
        for name in ignore {
            if name.eq(isit) {
                info!("{} vs {}", name, isit);
                return true;
            }
        }
        false
    }

    fn load(&self, parent: PathBuf, level: u32, ignore: &Vec<String>) {
        let dirs = fs::read_dir(parent).unwrap();

        // warn!("ID: {:?}", thread::current().id());
        for file in dirs {
            let ff = &file.unwrap().path();
            let buf = ff.to_owned();
            if Scan::is_ignore(&ignore, &buf) {
                continue;
            }

            if ff.is_dir() {
                // warn!("Dir: {}", ff.to_str().unwrap());
                self.dir_count.fetch_add(1, Ordering::Relaxed);
                self.send(FinderMsg::Dir(buf, level + 1));
            } else if ff.symlink_metadata().unwrap().file_type().is_symlink() {
                warn!("Synlink: {}", ff.to_str().unwrap());
            } else if ff.is_file() {
                // warn!("File: {}", ff.to_str().unwrap());
                self.file_count.fetch_add(1, Ordering::Relaxed);
                self.send(FinderMsg::File(buf, level));
            }
        }
        self.cnt.fetch_sub(1, Ordering::Relaxed);
    }

}

impl Actor for Scan {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        println!("Scan Actor is alive; {:?}", thread::current().name());
    }

    fn stopped(&mut self, ctx: &mut Context<Self>) {
        println!("Scan Actor is stopped; {:?}", thread::current().name());
    }
}

impl Handler<FinderMsg> for Scan {
    type Result = Result<Find, io::Error>;

    fn handle(&mut self, msg: FinderMsg, ctx: &mut Context<Self>) -> Self::Result {
        let mut list = Box::new(Vec::new());
        Ok(Find(list))
    }
}

#[derive(Debug)]
pub enum FinderMsg {
    Dir(PathBuf, u32),
    File(PathBuf, u32),
    Close(u32),
}

impl Message for FinderMsg {
    type Result = Result<Find, io::Error>;
}

struct Find(Box<Vec<FinderMsg>>);

impl Message for Find {
    type Result = Result<FinderMsg, io::Error>;
}

struct Finder;

impl Actor for Finder {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        println!("Actor is alive; {:?}", thread::current().name());
    }

    fn stopped(&mut self, ctx: &mut Context<Self>) {
        println!("Actor is stopped; {:?}", thread::current().name());
    }
}

impl Handler<FinderMsg> for Finder {
    type Result = Result<Find, io::Error>;

    fn handle(&mut self, msg: FinderMsg, ctx: &mut Context<Self>) -> Self::Result {
        println!("Find received; {:?}", thread::current().name());

        let mut list = Box::new(Vec::new());
        Ok(Find(list))
    }
}

pub fn run() {
    println!("start!" );
    System::run(|| {
        let addr = Finder.start();
        let res = addr.send(Find(FinderMsg::Close(1)));

        println!("wait1!" );

        tokio::spawn(
            res.map(|res| {
                println!("RESULT: {:?}; {:?}", res, thread::current().name());

                // stop system and exit
                // System::current().stop();
            }).map_err(|_| ()),
        );
    });

    let ten_millis = time::Duration::from_millis(1000);
    thread::sleep(ten_millis);
    println!("end!" );
}
