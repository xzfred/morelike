use std::thread::{self,JoinHandle};
use std::sync::mpsc::{channel, Sender, Receiver, RecvError};
use std::sync::{Arc, Mutex};

pub struct ThreadPool {
    state: Arc<PoolState>,
}

enum Message {
    Run(Task),
    Close,
}

struct PoolState {
    tx: Mutex<Sender<Message>>,
}
