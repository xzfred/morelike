#![allow(dead_code)]
#![allow(unused_must_use)]

use actix::prelude::*;
use futures::Future;
use tokio;
use std::{thread, time};
use ignore::WalkBuilder;
use ignore::overrides::{Override, OverrideBuilder};

use std::fs::{self};
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Scan {
    ignore: Vec<String>,
}

#[derive(Debug)]
pub struct Dir(PathBuf);

impl Message for Dir {
    type Result = Result<(), io::Error>;
}

#[derive(Debug)]
pub struct File(PathBuf);

impl Message for File {
    type Result = Result<(), io::Error>;
}

impl Scan {
    pub fn new(i: Vec<String>) -> Scan {
        Scan {
            ignore: i,
        }
    }
}

impl Actor for Scan {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Context<Self>) {
        debug!("Scan Actor is alive; {:?}", thread::current().name());
    }

    fn stopped(&mut self, _ctx: &mut Context<Self>) {
        debug!("Scan Actor is stopped; {:?}", thread::current().name());
    }
}

impl Handler<Dir> for Scan {
    type Result = Result<(), io::Error>;

    fn handle(&mut self, msg: Dir, _ctx: &mut Context<Self>) -> Self::Result {
        debug!("handle: {:?}; {:?}", msg, thread::current().name());

        let mut over_builder = OverrideBuilder::new(msg.0.to_path_buf());
        // let mut over_builder = OverrideBuilder::new("/");
        for i in &self.ignore {
            over_builder.add(i);
        }
        let ovr = over_builder.build().unwrap();
        // debug!("ignore:{:?}", ovr);
        // debug!("ignore:\n {:?},\n {:?}, \n{:?}, \n{:?}",
        //        ovr.matched(PathBuf::from(".git"), true),
        //        ovr.matched(PathBuf::from("./Library/Containers"), true),
        //        ovr.matched(PathBuf::from("/Users/xuzhi/Library/Containers"), true),
        //        ovr.matched(PathBuf::from("/Users/xuzhi/my/dev/morelike/target"), true));
        // return Ok(());

        let mut walker_builder = WalkBuilder::new(msg.0);
        walker_builder.threads(2).standard_filters(false).ignore(true);
        walker_builder.overrides(ovr);
        let walker = walker_builder.build_parallel();
        walker.run(|| {
            Box::new(move |result| {
                use ignore::WalkState::*;
                let entry = result.unwrap();
                if entry.file_type().unwrap().is_file() {
                    debug!("file: {:?}, {:?}", entry.path(), thread::current().id());
                } else {
                    debug!("dir: {:?}, {:?}", entry.path(), thread::current().id());
                }
                Continue
            })
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    extern crate log;
    extern crate pretty_env_logger;
    extern crate actix;
    extern crate futures;
    extern crate tokio;
    use actix::prelude::*;
    use futures::Future;
    // use tokio;
    use std::{thread, time};
    use std::path::{Path, PathBuf};
    use scan::*;

    #[test]
    fn test_scan_actor() {
        pretty_env_logger::init();
        System::run(|| {
            let addr = Scan::new(vec![
                "!.git".to_owned(),
                "!target".to_owned(),
                "!**/Library/Containers/**".to_owned(),
            ]).start();
            let res = addr.send(Dir(PathBuf::from(".")));

            tokio::spawn(
                res.map(|res| {
                    debug!("RESULT: {:?}; {:?}", res, thread::current().name());

                    match res {
                        Ok(r) => assert_eq!(r, ()),
                        Err(_err) => {},
                    }
                    // stop system and exit
                    System::current().stop();
                }).map_err(|_| ()),
            );
        });
    }
}
