use std::net::TcpListener;

// Custom
mod handler;
mod renderer;
mod requests;
mod threads;

// For convenience
use requests::RequestResult;
use threads::{ThreadPool, ThreadPoolError};

fn main() {
    const BIND_PORT: &str = "7878";
    const BASE_PATH: &str = "/home/leah";
    let listener = TcpListener::bind("127.0.0.1:".to_owned() + BIND_PORT).unwrap();

    println!(
        "[INFO] {}",
        "Serving files on http://localhost:".to_owned() + BIND_PORT + "/fs"
    );

    let pool_size: usize = 4;
    let thread_pool = ThreadPool::build(pool_size);
    if thread_pool.is_err() {
        println!("[ERR] Could not create threadpool with {pool_size} threads. Aborting");
    }
    let thread_pool = thread_pool.unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        // Handle stream in threadpool
        let stream_handle = thread_pool.execute(|| {
            let handle_result = handler::handle_connection(stream, BASE_PATH);

            // If the handling of the request fails, deal with it
            if handle_result.is_ok() {
                if let RequestResult::Ok(ok_msg) = handle_result.unwrap() {
                    println!("[REQ] {ok_msg}")
                }
            } else {
                print!("[ERR] ");
                match handle_result.unwrap_err() {
                    RequestResult::UnsupportedURI(uri) => {
                        println!("URI '{uri}' is not currently supported")
                    }
                    RequestResult::InvalidRequest => println!("Invalid request"),
                    RequestResult::InvalidMethod => println!("HTTP method not supported"),
                    RequestResult::FileNotFound(err)
                    | RequestResult::RenderingError(err)
                    | RequestResult::StreamError(err) => println!("{err}"),
                    RequestResult::FilePathNotFound => {
                        println!("Path specified in the url was not found")
                    }
                    _ => println!("Request error"),
                }
            }
        });

        // Deal with threadpool error
        if let Err(ThreadPoolError::ClosureExecError(msg)) = stream_handle {
            println!("[ERR] {msg}");
        }
    }

    println!("Shutdown complete!");
}
