pub mod renderer;

#[derive(Debug)]
pub enum RequestResult {
    Ok(String),
    UnsupportedURI(String),
    InvalidRequest,
    InvalidMethod,
    StreamError(String),
    FileNotFound(String),
    FilePathNotFound,
}

// use std::thread;

// pub enum PoolCreationError{
//     InvalidSize,
// }

// pub struct ThreadPool{
//     threads: Vec<thread::JoinHandle<()>>
// }

// impl ThreadPool{
//     // /// Create a new ThreadPool.
//     // ///
//     // /// The size is the number of threads in the pool.
//     // ///
//     // /// # Panics
//     // ///
//     // /// The `new` function will panic if the size is zero.
//     // pub fn new(size: usize) -> ThreadPool{
//     //     assert!(size > 0);

//     //     ThreadPool
//     // }

//     /// Builds a new ThreadPool.
//     ///
//     /// The size is the number of threads in the pool.
//     ///
//     /// This does not panic
//     pub fn build(size: usize) -> Result<ThreadPool, PoolCreationError>{
//         if size < 1{
//             return Err(PoolCreationError::InvalidSize);
//         }

//         let mut threads = Vec::with_capacity(size);

//         for i in 0..size{
//             // threads[i] = thread::spawn(f)
//         }

//         Ok(ThreadPool{threads})
//     }

//     pub fn execute<F>(&self, f: F)
//     where
//         F: FnOnce() + Send + 'static,
//     {
//     }
// }
