use log;
use simple_logger::SimpleLogger;
use std::{net::{TcpListener, TcpStream}, num::NonZero, sync::mpsc, thread};
use time::macros::format_description;
use whoami;

// Custom
mod handler;
mod threads;
mod renderer;
mod requests;

// For convenience
use requests::RequestResult;
use threads::{ThreadPool, ThreadPoolError};

const BIND_PORT: &str = "7878";

fn main(){
    SimpleLogger::new()
        .with_level(log::LevelFilter::Trace)
        .env()
        .with_timestamp_format(format_description!("[year]-[month]-[day] [hour]:[minute]:[second]"))
        .init()
        .unwrap();

    let base_path: String = format!("/home/{}", whoami::username());

    let listener = TcpListener::bind("127.0.0.1:".to_owned() + BIND_PORT).unwrap();

    log::info!(
        "{}",
        "Serving files on http://localhost:".to_owned() + BIND_PORT + "/fs"
    );

    let pool_size = thread::available_parallelism()
        .unwrap_or_else(|_|{
            log::warn!("Could not obtain available parallelism. Defaulting to 1");
            NonZero::new(1).expect("Could not default to 1. Aborting...")
        })
        .get();

    let thread_pool = ThreadPool::build(pool_size);
    if thread_pool.is_err() {
        log::error!("Could not create threadpool with {pool_size} threads. Aborting");
    }
    let thread_pool = thread_pool.unwrap();

    // Setup for Ctrl-C signal
    let (tx, rx) = mpsc::channel();
    ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
        .expect("Error setting Ctrl-C handler");


    listener.set_nonblocking(true).expect("Cannot set non-blocking");
    loop{        
        if let Ok((stream, _)) = listener.accept(){
            execute_request(&thread_pool, stream, base_path.clone());           
        }

        // Check for Ctrl-C signal
        if rx.try_recv().is_ok(){
            log::info!("Ctrl-C signal received. Exiting...");
            break;
        }
    }
}

fn execute_request(thread_pool: &ThreadPool, stream_request: TcpStream, base_path: String){    
    // Handle stream in threadpool
    let stream_handle = thread_pool.execute(|| {
        let handle_result = handler::handle_connection(stream_request, base_path);

        // If the handling of the request fails, deal with it
        if handle_result.is_ok() {
            if let RequestResult::Ok(ok_msg) = handle_result.unwrap() {
                log::debug!("{ok_msg}")
            }
        } else {
            match handle_result.unwrap_err() {
                RequestResult::UnsupportedURI(uri) => {
                    log::error!("URI '{uri}' is not currently supported")
                }
                RequestResult::InvalidRequest => log::error!("Invalid request"),
                RequestResult::InvalidMethod => log::error!("HTTP method not supported"),
                RequestResult::FileNotFound(err)
                | RequestResult::RenderingError(err)
                | RequestResult::StreamError(err) => log::error!("{err}"),
                RequestResult::FilePathNotFound => {
                    log::error!("Path specified in the url was not found")
                }
                _ => log::error!("Request error"),
            }
        }
    });

    // Deal with threadpool error
    if let Err(ThreadPoolError::ClosureExecError(msg)) = stream_handle {
        log::error!("{msg}");
    }
}