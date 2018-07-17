use std::fs::{self};
use std::io;
use std::path::{Path, PathBuf};

use actix::prelude::*;
use futures::Future;
use tokio;

use std::{thread, time};

#[derive(Debug)]
pub enum FinderMsg {
    Dir(PathBuf, u32),
    File(PathBuf, u32),
    Close(u32),
}

struct Find(FinderMsg);

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

impl Handler<Find> for Finder {
    type Result = Result<FinderMsg, io::Error>;

    fn handle(&mut self, msg: Find, ctx: &mut Context<Self>) -> Self::Result {
        println!("Find received; {:?}", thread::current().name());

        println!("wait2!" );
        let ten_millis = time::Duration::from_millis(1000);
        thread::sleep(ten_millis);

        Ok(msg.0)
    }
}

pub fn run() {
    println!("start!" );
    System::run(|| {
        let addr = Finder.start();
        let res = addr.send(Find(FinderMsg::Close(1)));
        let res1 = addr.send(Find(FinderMsg::Close(2)));

        let addr1 = Finder.start();
        let res2 = addr1.send(Find(FinderMsg::Close(3)));

        println!("wait1!" );
        let ten_millis = time::Duration::from_millis(1000);
        thread::sleep(ten_millis);
        tokio::spawn(
            res.map(|res| {
                println!("RESULT: {:?}; {:?}", res, thread::current().name());

                // stop system and exit
                // System::current().stop();
            }).map_err(|_| ()),
        );

        tokio::spawn(
            res1.map(|res| {
                println!("RESULT: {:?}; {:?}", res, thread::current().name());

                // stop system and exit
                // System::current().stop();
            }).map_err(|_| ()),
        );

        tokio::spawn(
            res2.map(|res| {
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
