use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

#[derive(Debug)]
pub enum ThreadPoolError {
    InvalidSize,
    ClosureExecError(String),
}

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    sender: Option<mpsc::Sender<Job>>,
    threads: Vec<Option<thread::JoinHandle<()>>>,
}

impl ThreadPool {
    /// Builds a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// This does not panic
    pub fn build(size: usize) -> Result<ThreadPool, ThreadPoolError> {
        if size < 1 {
            return Err(ThreadPoolError::InvalidSize);
        }

        let mut threads = Vec::with_capacity(size);

        // Create a channel and wrap receiver in mutex and arc.
        // An arc clone will be passed to each new thread.
        // This will have a single producer and multiple consumers
        let (tx, rx) = mpsc::channel::<Job>();
        let receiver = Arc::new(Mutex::new(rx));

        for i in 0..size {
            let rec_clone = Arc::clone(&receiver);
            threads.push(Some(thread::spawn(move || loop {
                let mutex = rec_clone.lock();
                if mutex.is_err() {
                    continue;
                }

                let message_result = mutex.unwrap().recv();

                match message_result {
                    Ok(job) => {
                        println!("[THREAD] Thread {i} got a job! Executing...");
                        job();
                    }
                    Err(_) => {
                        println!("[THREAD] Thread {i} disconnected.");
                        break;
                    }
                }
            })));
        }

        Ok(ThreadPool {
            sender: Some(tx),
            threads,
        })
    }

    /// Executes the given closure in a thread
    ///  
    pub fn execute<F>(&self, f: F) -> Result<(), ThreadPoolError>
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).map_err(|_e| {
            ThreadPoolError::ClosureExecError("Closure could not be send to a thread".to_string())
        })
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Close down the channel so no other requests are accepted
        drop(self.sender.take());

        // Join the threads
        for thread in &mut self.threads {
            if let Some(t) = thread.take() {
                t.join().unwrap();
            }
        }
    }
}
