use simple_logger::SimpleLogger;
use time::macros::format_description;

// Custom
mod handler;
mod renderer;
mod requests;

// For convenience
use requests::RequestError;
use tokio::net::{TcpListener, TcpStream};

const BIND_PORT: &str = "7878";

#[tokio::main]
async fn main() {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Trace)
        .env()
        .with_timestamp_format(format_description!(
            "[year]-[month]-[day] [hour]:[minute]:[second]"
        ))
        .init()
        .unwrap();

    let base_path: String = format!("/home/{}", whoami::username());

    let listener = TcpListener::bind("127.0.0.1:".to_owned() + BIND_PORT)
        .await
        .unwrap();

    log::info!(
        "{}",
        "Serving files on http://localhost:".to_owned() + BIND_PORT + "/fs"
    );

    loop {
        if let Ok((socket, _)) = listener.accept().await {
            let base_path_clone = base_path.clone();
            tokio::spawn(async move {
                log::info!("Request received!");
                execute_and_log_request(socket, base_path_clone).await;
            });
        }
    }
}

async fn execute_and_log_request(stream_request: TcpStream, base_path: String) {
    let handle_result = handler::handle_connection(stream_request, base_path).await;

    // If the handling of the request fails, deal with it
    match handle_result {
        Ok(msg) => log::debug!("{msg}"),
        Err(error) => match error {
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
        },
    }
}
