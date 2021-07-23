use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use chrono::{DateTime, Utc};
pub use ansi_term::Colour;
use std::env;
use lazy_static::lazy_static;


lazy_static! {
    pub static ref LOG_LEVEL: String = env::var("LOGLEVEL").unwrap_or(String::from("INFO"));
}


#[macro_export]
macro_rules! log {
    ($level: expr, $text: expr) => {

        assert!($level == "debug" || $level == "info" || $level == "warning" || $level == "error");
        let now: DateTime<Utc> = Utc::now();
        let formatted = String::from(format!("[{}] {} [{}:{}]: {}", $level, now.format("%Y-%m-%d %H:%M:%S"), file!(), line!(), $text));

        if *LOG_LEVEL == "DEBUG" {
            let val = formatted.to_owned();
            match $level {
                "debug" => println!("{}", Colour::Green.paint(val)),
                "info" => println!("{}", val),
                "warning" => println!("{}", Colour::Yellow.paint(val)),
                "error" => println!("{}", Colour::Red.paint(val)),
                _ => {},
            }
        };

        if *LOG_LEVEL == "INFO" {
            let val = formatted.to_owned();
            match $level {
                "info" => println!("{}", val),
                "warning" => println!("{}", Colour::Yellow.paint(val)),
                "error" => println!("{}", Colour::Red.paint(val)),
                _ => {},
            }
        };

        if *LOG_LEVEL == "WARNING" {
            let val = formatted.to_owned();
            match $level {
                "warning" => println!("{}", Colour::Yellow.paint(val)),
                "error" => println!("{}", Colour::Red.paint(val)),                _ => {},
            }
        };

        if *LOG_LEVEL == "ERROR" {
            let val = formatted.to_owned();
            match $level {
                "error" => println!("{}", Colour::Red.paint(val)),
                _ => {},
            }
        };
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is 0.

    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        log!("debug", "Sending terminate message to all workers.");

        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        log!("debug", "Shutting down all workers.");

        for worker in &mut self.workers {
            log!("debug", format!("Shutting down worker: {}", worker.id));

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}


struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}


impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();

            match message {
                Message::NewJob(job) => {
                    log!("debug", format!("Worker got a new job, id: {}", id));
                    job();
                }
                Message::Terminate => {
                    log!("debug", format!("Worker was told to terminate, id: {}", id));
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}
