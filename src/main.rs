use std::{net::{TcpListener, TcpStream}, sync::mpsc, thread, time::Duration};
use ctrlc;

// Custom
mod handler;
mod renderer;
mod requests;
mod threads;

// For convenience
use requests::RequestResult;
use threads::{ThreadPool, ThreadPoolError};

const BIND_PORT: &str = "7878";
const BASE_PATH: &str = "/home/leah";

fn main() {
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

    // Setup for Ctrl-C signal
    let (tx, rx) = mpsc::channel();
    ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
        .expect("Error setting Ctrl-C handler");


    listener.set_nonblocking(true).expect("Cannot set non-blocking");
    loop{        
        if let Ok((stream, _)) = listener.accept(){
            execute_request(&thread_pool, stream);           
        }

        // Check for Ctrl-C signal
        if let Ok(_) = rx.try_recv(){
            println!("[SYS] Ctrl-C signal received. Exiting...");
            break;
        }

        // Prevents from overusing cpu time
        thread::sleep(Duration::from_millis(250));
    }
}

fn execute_request(thread_pool: &ThreadPool, stream_request: TcpStream){
    // Handle stream in threadpool
    let stream_handle = thread_pool.execute(|| {
        let handle_result = handler::handle_connection(stream_request, BASE_PATH);

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