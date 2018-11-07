#[allow(unused_imports)]

use std::path::{Path, PathBuf};
// use std::io;
// use std::fs;

#[macro_use] extern crate log;
extern crate pretty_env_logger;
extern crate tokio_fs;
extern crate tokio;
extern crate futures;

use futures::{Future, Stream};
// use tokio_fs::{ReadDir};
use std::{thread, time};

extern crate mach;
extern crate libc;

use libc::*;
use mach::traps::*;
use mach::task::*;
// use mach::task_info::*;
// use mach::kern_return::*;
use mach::port::*;
use mach::mach_types::*;
use mach::message::*;

fn getThreadNum() {
    // let num: isize = 0;
    let pid = std::process::id();

    // let mut r: kern_return_t;


    unsafe {
        let mut task: mach_port_name_t = 0;
        let mut thread_list: thread_act_array_t = 0 as thread_act_array_t;
        let mut thread_count: mach_msg_type_number_t = 0;
        task_for_pid(
            mach_task_self(),
            pid as c_int,
            &mut task as *mut mach_port_name_t);

        task_threads(task,
                         &mut thread_list as *mut *mut u32,
                         &mut thread_count as *mut u32);

        println!("thread num: {:?}===========================", thread_count);
    }
}

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
        println!("thread id: {:?}", thread::current().id());
        thread::sleep(ten_millis);
        Ok(())
    }).map_err(|err| {
        eprintln!("Error: {:?}", err);
        ()
    });
    tokio::run(fut);
    getThreadNum();
}
