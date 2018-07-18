use std::fs::{self};
use std::io;
use std::path::{Path, PathBuf};

use actix::prelude::*;
use futures::Future;
use tokio;

use std::{thread, time};
use Scan::*;

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
