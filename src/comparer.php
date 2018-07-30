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

pub struct Comparer {
}
