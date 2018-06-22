use std::thread::{self,JoinHandle};
use std::sync::mpsc::{channel, Sender, Receiver, RecvError};
use std::sync::{Arc, Mutex};

/// 线程池消息
pub enum PoolMsg<T> {
    Start,
    Wait,
    Task(T),
    End,
}

/// 线程池
pub struct Pool<T> {
    size: u32,
    state: PoolState<T>,
}

/// 线程池状态
pub struct PoolState<T> {
    id: u32,
    rx: Mutex<Receiver<PoolMsg<T>>>,
    tx: Mutex<Sender<PoolMsg<T>>>,
}
