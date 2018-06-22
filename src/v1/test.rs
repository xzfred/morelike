// use std::env;
// use std::fs::{self, DirEntry, File, ReadDir};
// use std::io::prelude::*;
// use std::path::Path;

// fn main() {
//     let args: Vec<String> = env::args().collect();
//     for param in args.iter() {
//         println!("{}", param);
//     }

//     let dirs = fs::read_dir(Path::new(".")).unwrap();

//     for f in dirs {
//         println!("{}", f.unwrap().path().to_str().unwrap());
//     }
// }
// #[warn(unused_imports)] #![feature(rustc_private)] extern crate rand;
// extern crate rustsync;
// use rustsync::*;
// use rand::Rng;

#![feature(test)]

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
extern crate pretty_env_logger;
#[macro_use] extern crate log;
extern crate adler32;

extern crate test;
use test::Bencher;

extern crate crc32c_hw;

use std::convert::AsRef;

fn main() {
    pretty_env_logger::init();

    // println!("{:?}", buf[..].to_vec());
}

fn read_to_buf(buf: &mut [u8]) {
    let path = Path::new("/Users/xuzhi/Music/虾米音乐/Beyond-不再犹豫.mp3");
    let display = path.display();
    info!("{}", display);
    let mut file: File = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}",
                           display,
                           why.description() ),
        Ok(file) => file,
    };
    file.read_exact(buf).unwrap();
}

#[bench]
fn bench_crc32_hw(b: &mut Bencher) {
    let mut buf: [u8; 1024] = [0; 1024];
    read_to_buf(&mut buf);
    b.iter(|| {
        for _i in 1..100 {
            crc32c_hw::compute(buf.as_ref());
        }
    })
}

#[bench]
fn bench_adler32(b: &mut Bencher) {
    let mut buf: [u8; 1024] = [0; 1024];
    read_to_buf(&mut buf);

    // println!("{}", hash.hash());
    b.iter(|| {
        for _i in 1..100 {
            let hash = adler32::RollingAdler32::from_buffer(&buf);
            hash.hash();
        }
    })
}

// fn main() {
//   // Create 4 different random strings first.
//   let chunk_size = 1000;
//   let a = rand::thread_rng()
//           .gen_ascii_chars()
//           .take(chunk_size)
//           .collect::<String>();
//   let b = rand::thread_rng()
//           .gen_ascii_chars()
//           .take(50)
//           .collect::<String>();
//   let b_ = rand::thread_rng()
//           .gen_ascii_chars()
//           .take(100)
//           .collect::<String>();
//   let c = rand::thread_rng()
//           .gen_ascii_chars()
//           .take(chunk_size)
//           .collect::<String>();

//   // Now concatenate them in two different ways.

//   let mut source = a.clone() + &b + &c;
//   let mut modified = a + &b_ + &c;

//   // Suppose we want to download `modified`, and we already have
//   // `source`, which only differs by a few characters in the
//   // middle.

//   // We first have to choose a block size, which will be recorded
//   // in the signature below. Blocks should normally be much bigger
//   // than this in order to be efficient on large files.

//   let block = [0; 32];

//   // We then create a signature of `source`, to be uploaded to the
//   // remote machine. Signatures are typically much smaller than
//   // files, with just a few bytes per block.

//   let source_sig = signature(source.as_bytes(), block).unwrap();

//   // Then, we let the server compare our signature with their
//   // version.

//   let comp = compare(&source_sig, modified.as_bytes(), block).unwrap();

//   // We finally download the result of that comparison, and
//   // restore their file from that.

//   let mut restored = Vec::new();
//   restore_seek(&mut restored, std::io::Cursor::new(source.as_bytes()), vec![0; 1000], &comp).unwrap();
//   assert_eq!(&restored[..], modified.as_bytes())
// }



















