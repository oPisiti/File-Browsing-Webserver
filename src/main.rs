use std::net::TcpListener;

// Custom 
mod handler;
mod renderer;
mod requests;
mod threads;

// For convenience
use requests::RequestResult;

fn main() {
    const BIND_ADDR: &str = "127.0.0.1:7878";
    let listener = TcpListener::bind(BIND_ADDR).unwrap();

    println!("Serving files on http://localhost:7878/fs");

    let base_path = "/home/leah";
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        let handle_result = handler::handle_connection(stream, base_path);

        // If the handling of the request fails, deal with it
        if handle_result.is_ok() {
            if let RequestResult::Ok(ok_msg) = handle_result.unwrap() {
                println!("{ok_msg}")
            }
        } else {
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
                _ => (),
            }
        }
    }
}

