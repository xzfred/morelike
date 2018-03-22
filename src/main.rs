#[allow(unused_imports)]

extern crate sha1;
extern crate pretty_env_logger;
#[macro_use] extern crate log;

mod file;

use file::{FileTable};

fn main() {
    pretty_env_logger::init();
    info!("start");
    let table = FileTable::new();
    table.scan(".");
}
