use crate::run;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use tokio::time::sleep;

pub struct Server {
    pub started: AtomicBool,
}

impl Server {
    pub fn new() -> Server {
        Server {
            started: AtomicBool::new(false),
        }
    }

    pub async fn init_server(&mut self) {
        if !self.started.load(Ordering::Relaxed) {
            thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("runtime starts");
                rt.spawn(run());
                loop {
                    thread::sleep(Duration::from_millis(100_000));
                }
            });
            sleep(Duration::from_millis(100)).await;
            self.started.store(true, Ordering::Relaxed);
        }
    }
}
