use simple_logger::SimpleLogger;
use time::macros::format_description;

// Custom
mod handler;
mod renderer;
mod requests;

// For convenience
use tokio::net::{TcpListener, TcpStream};
use requests::RequestError;

const BIND_PORT: &str = "7878";

#[tokio::main]
async fn main(){
    SimpleLogger::new()
        .with_level(log::LevelFilter::Trace)
        .env()
        .with_timestamp_format(format_description!("[year]-[month]-[day] [hour]:[minute]:[second]"))
        .init()
        .unwrap();

    let base_path: String = format!("/home/{}", whoami::username());

    let listener = TcpListener::bind("127.0.0.1:".to_owned() + BIND_PORT).await.unwrap();

    log::info!(
        "{}",
        "Serving files on http://localhost:".to_owned() + BIND_PORT + "/fs"
    );

    loop{
        if let Ok((socket, _)) = listener.accept().await{
            tokio::spawn(execute_request(socket, base_path.clone()));
        }
    }
    // log::info!("Awaiting ctrl-c...");
    // let a = tokio::signal::ctrl_c().await;

    // let pool_size: usize = 4;
    // let thread_pool = ThreadPool::build(pool_size);
    // if thread_pool.is_err() {
    //     log::error!("Could not create    threadpool with {pool_size} threads. Aborting");
    // }
    // let thread_pool = thread_pool.unwrap();

    // // Setup for Ctrl-C signal
    // let (tx, rx) = mpsc::channel();
    // ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
    //     .expect("Error setting Ctrl-C handler");

    // listener.set_nonblocking(true).expect("Cannot set non-blocking");
    // loop{        
    //     if let Ok((stream, _)) = listener.accept(){
    //         execute_request(&thread_pool, stream, base_path.clone());           
    //     }

    //     // Check for Ctrl-C signal
    //     if rx.try_recv().is_ok(){
    //         log::info!("Ctrl-C signal received. Exiting...");
    //         break;
    //     }

    //     // Prevents from overusing cpu time
    //     thread::sleep(Duration::from_millis(250));
    // }    
}

async fn execute_request(stream_request: TcpStream, base_path: String){    
    let handle_result = handler::handle_connection(stream_request, base_path).await;

    // If the handling of the request fails, deal with it
    if handle_result.is_ok() {
        let ok_msg = handle_result.unwrap();
        log::debug!("{ok_msg}");
    } else {
        match handle_result.unwrap_err() {
            RequestError::UnsupportedURI(uri) => {
                log::error!("URI '{uri}' is not currently supported")
            }
            RequestError::InvalidRequest => log::error!("Invalid request"),
            RequestError::InvalidMethod => log::error!("HTTP method not supported"),
            RequestError::FileNotFound(err)
            | RequestError::RenderingError(err)
            | RequestError::StreamError(err) => log::error!("{err}"),
            RequestError::FilePathNotFound => {
                log::error!("Path specified in the url was not found")
            }
        }
    }
}
