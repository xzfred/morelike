
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};
use std::sync::mpsc;
use std::{thread, time};

/// 线程池消息
pub enum Message {
    Run(Task),
    Close,
}

/// 线程池
pub struct ThreadPool {
    state: Arc<PoolState>,
}

pub struct ThreadPoolBuilder {
    pool_size: usize,
    stack_size: usize,
    name_prefix: Option<String>,
    after_start: Option<Arc<Fn(usize) + Send + Sync>>,
    before_stop: Option<Arc<Fn(usize) + Send + Sync>>,
}

pub struct WaitPool {
    active_size: AtomicUsize,
}

impl WaitPool {
    pub fn new() -> WaitPool {
        WaitPool {
            active_size: AtomicUsize::new(0),
        }
    }

    pub fn enter(&self) {
        let idx = self.active_size.fetch_add(1, Ordering::Relaxed);
        info!("enter:{}", idx);
    }

    pub fn leave(&self) {
        let idx = self.active_size.fetch_sub(1, Ordering::Relaxed);
        info!("leave:{}", idx);
    }

    pub fn join(&self) {
        loop {
            if self.active_size.load(Ordering::Relaxed) > 0 {
                let ten_millis = time::Duration::from_millis(10);
                thread::sleep(ten_millis);
                info!("join...");
            } else {
                break;
            }
        }
    }
}

/// 线程池状态
struct PoolState {
    rx: Mutex<mpsc::Receiver<Message>>,
    tx: Mutex<mpsc::Sender<Message>>,
    cnt: AtomicUsize,
    // active_size: Arc<AtomicUsize>,
    size: usize,
}

impl PoolState {
    fn send(&self, msg: Message) {
        self.tx.lock().unwrap().send(msg).unwrap();
    }

    fn work(&self,
            idx: usize,
            after_start: Option<Arc<Fn(usize) + Send + Sync>>,
            before_stop: Option<Arc<Fn(usize) + Send + Sync>>) {
        // let _scope = enter().unwrap();
        after_start.map(|fun| fun(idx));
        loop {
            let msg = self.rx.lock().unwrap().recv().unwrap();
            match msg {
                Message::Run(r) => r.run(),
                Message::Close => break,
            }
        }
        before_stop.map(|fun| fun(idx));
    }
}

impl ThreadPool {
    pub fn new() -> ThreadPool {
        ThreadPoolBuilder::new().create()
    }

    pub fn builder() -> ThreadPoolBuilder {
        ThreadPoolBuilder::new()
    }

    pub fn spawn<F>(&self, f: F)-> & ThreadPool where
        F: Fn() + Send + Sync + 'static
    {
        let task = Task {
            spawn: Some(Arc::new(f)),
        };
        self.state.send(Message::Run(task));
        self
    }
}

impl Clone for ThreadPool {
    fn clone(&self) -> ThreadPool {
        self.state.cnt.fetch_add(1, Ordering::Relaxed);
        ThreadPool { state: self.state.clone() }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        if self.state.cnt.fetch_sub(1, Ordering::Relaxed) == 1 {
            for _ in 0..self.state.size {
                self.state.send(Message::Close);
            }
        }
    }
}

impl ThreadPoolBuilder {
    pub fn new() -> ThreadPoolBuilder {
        ThreadPoolBuilder {
            pool_size: 2,
            stack_size: 0,
            name_prefix: None,
            after_start: None,
            before_stop: None,
        }
    }

    pub fn pool_size(&mut self, size: usize) -> &mut Self {
        self.pool_size = size;
        self
    }

    pub fn stack_size(&mut self, stack_size: usize) -> &mut Self {
        self.stack_size = stack_size;
        self
    }

    pub fn name_prefix<S: Into<String>>(&mut self, name_prefix: S) -> &mut Self {
        self.name_prefix = Some(name_prefix.into());
        self
    }

    pub fn after_start<F>(&mut self, f: F) -> &mut Self
        where F: Fn(usize) + Send + Sync + 'static
    {
        self.after_start = Some(Arc::new(f));
        self
    }

    pub fn before_stop<F>(&mut self, f: F) -> &mut Self
        where F: Fn(usize) + Send + Sync + 'static
    {
        self.before_stop = Some(Arc::new(f));
        self
    }

    pub fn create(&mut self) -> ThreadPool {
        let (tx, rx) = mpsc::channel();
        let pool = ThreadPool {
            state: Arc::new(PoolState {
                tx: Mutex::new(tx),
                rx: Mutex::new(rx),
                cnt: AtomicUsize::new(1),
                // active_size: Arc::new(AtomicUsize::new(self.pool_size)),
                size: self.pool_size,
            }),
        };
        assert!(self.pool_size > 0);

        for counter in 0..self.pool_size {
            let state = pool.state.clone();
            let after_start = self.after_start.clone();
            let before_stop = self.before_stop.clone();
            let mut thread_builder = thread::Builder::new();
            if let Some(ref name_prefix) = self.name_prefix {
                thread_builder = thread_builder.name(format!("{}{}", name_prefix, counter));
            }
            if self.stack_size > 0 {
                thread_builder = thread_builder.stack_size(self.stack_size);
            }
            thread_builder.spawn(move || state.work(counter, after_start, before_stop)).unwrap();
        }
        return pool
    }
}

pub struct Task {
    spawn: Option<Arc<Fn() + Send + Sync>>, 
    // spawn: Box<Future<Item = (), Error = Never> + Send>,
    // map: LocalMap,
    // exec: ThreadPool,
    // wake_handle: Arc<WakeHandle>,
}

impl Task {
    /// Actually run the task (invoking `poll` on its future) on the current
    /// thread.
    pub fn run(&self) {
        let fun = self.spawn.clone();
        fun.map(|fun| fun());
    }
}
