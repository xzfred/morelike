use std::thread::{self,JoinHandle};
use std::sync::mpsc::{channel, Sender, Receiver, RecvError};
use std::sync::{Arc, Mutex};

struct Pool{
    state: Arc<WorkState>,
}

enum WorkMsg {
    Begin(u32, u32),
    Wait(u32, u32),
    Work(u32, u32),
    Exit(u32, u32),
}

struct WorkState<T> {
    size: u32,
    rx: Mutex<Receiver<Option<T>>>,
    tx: Mutex<Sender<Option<T>>>,
    pos_tx: Mutex<Sender<P>>,
}

impl<T> WorkState<T> {
    pub fn new(pos_tx: Sender<P>, size: u32) -> WorkState<T, P> {
        let (sender, receiver) = channel::<T>();
        WorkState {
            size,
            pos_tx: Mutex::new(pos_tx),
            rx: Mutex::new(receiver),
            tx: Mutex::new(sender),
        }
    }

    pub fn send(&self, msg: T) {
        self.tx.lock().unwrap().send(Some(msg)).unwrap();
    }

    pub fn send_pos(&self, msg: P) {
        self.pos_tx.lock().unwrap().send(msg).unwrap();
    }

    pub fn run(&self, id: u32) {
        self.send_pos(WorkState::Begin(id, self.size));
        loop {
            let msg = self.rx.lock().unwrap().recv().unwrap();
            match msg {
            }

        }
        self.send_pos(WorkState::Exit(id, self.size));
    }
}

impl Pool {
    pub fn new(size: u32) {
        let state = Arc::new(WorkState::new());
    }
}
