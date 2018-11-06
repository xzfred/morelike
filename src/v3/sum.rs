#![allow(unused_must_use)]

use std::io::{Read};
use std::fs::{File};
use std::hash::{Hasher, Hash};
use std::io;
use std::{thread, time};
use std::path::{Path, PathBuf};

use twox_hash::{XxHash};

use actix::prelude::*;
use futures::Future;
use tokio;

use scan::*;

const BUFSIZE: usize = 1024;

// type Output<N> = GenericArray<u8, N>;
// static DEFAULT_SUM: [u8; 20] = [0; 20];

type HashSum = u64;

pub fn checksum(path: &PathBuf, all: bool) -> io::Result<HashSum> {
    let mut file: File = File::open(path)?;

    let mut buf: [u8; BUFSIZE] = [0; BUFSIZE];

    // let mut handle = file.take(BUFSIZE as u64);
    let mut hash = XxHash::with_seed(0);
    loop {
        let read_size = file.read(&mut buf)?;
        if read_size > 0 {
            hash.write(&buf.as_ref());
            if !all {
                break;
            }
        } else {
            break;
        }
    }
    let sum = hash.finish();
    Ok(sum)
}

#[derive(Debug)]
pub struct Sum {

}

#[derive(Debug)]
pub struct DpFile(PathBuf, bool);

impl DpFile {
    pub fn new(path: PathBuf, all: bool) -> DpFile {
        DpFile(path, all)
    }
}

impl Message for DpFile {
    type Result = Result<HashSum, io::Error>;
}

impl Actor for Sum {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Context<Self>) {
        debug!("Sum Actor is alive; {:?}", thread::current().name());
    }

    fn stopped(&mut self, _ctx: &mut Context<Self>) {
        debug!("Sum Actor is stopped; {:?}", thread::current().name());
    }
}

impl Handler<DpFile> for Sum {
    type Result = Result<HashSum, io::Error>;

    fn handle(&mut self, msg: DpFile, _ctx: &mut Context<Self>) -> Self::Result {
        debug!("File handle: {:?}; {:?}", msg, thread::current().name());

        // match checksum(&msg.0, false) {
        //     Ok(sum) => Ok(sum),
        //     Err(e) => Err(e),
        // }
        checksum(&msg.0, msg.1)
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
    use std::{thread, time};
    use std::path::{Path, PathBuf};
    use sum::*;

    #[test]
    fn test_sum_actor() {
        // pretty_env_logger::init();
        System::run(|| {
            let addr = Sum{}.start();
            let res = addr.send(DpFile(PathBuf::from("./test/u28.png"), false));
            tokio::spawn(
                res.map(|res| {
                    debug!("RESULT: {:?}; {:?}", res, thread::current().name());

                    match res {
                        Ok(r) => assert_eq!(r, 2086477930382796716),
                        Err(_err) => {},
                    }
                    // stop system and exit
                    System::current().stop();
                }).map_err(|_| ()),
            );

            let res1 = addr.send(DpFile(PathBuf::from("./test/u28.png"), true));
            tokio::spawn(
                res1.map(|res| {
                    debug!("RESULT: {:?}; {:?}", res, thread::current().name());

                    match res {
                        Ok(r) => assert_eq!(r, 7733030760293779963),
                        Err(_err) => {},
                    }
                    // stop system and exit
                    System::current().stop();
                }).map_err(|_| ()),
            );
        });
    }
}
