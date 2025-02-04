use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::time;
use tokio::fs;

// Custom
use crate::renderer;

// For convenience
use crate::renderer::RenderError;
use crate::requests::RequestError;

pub async fn handle_connection(   
    mut stream: TcpStream,
    base_path: String,
) -> Result<String, RequestError> {

    let pages_path = String::from("pages/");
    let mut buf_reader = BufReader::new(&mut stream);

    const FS_ID: &str = "/fs";

    // Get the http header tokens
    let mut request_line = String::new();
    buf_reader        
        .read_line(&mut request_line)
        .await
        .map_err(|_| RequestError::StreamError("Could not read stream buffer".to_string()))?;
    let request_tokens = request_line.split_whitespace().collect::<Vec<&str>>();

    // Check for valid method
    match request_tokens[0] {
        "GET" => (),
        _ => return Err(RequestError::InvalidMethod),
    }

    // URI not present
    if request_tokens.len() < 2 {
        return Err(RequestError::InvalidRequest);
    }

    // Handle URI request
    let uri = request_tokens[1];
    let mut status_line: String = "HTTP/1.1 200 OK".to_string();
    let mut file_name: String = pages_path.clone();
    let mut is_valid_uri = true;
    let mut is_static_page = true;
    let mut render_flags = renderer::RenderFlags::default();
    match uri {
        "/" => file_name += "root.html",
        "/flowers" => file_name += "flowers.html",
        "/sleep" => {
            time::sleep(time::Duration::from_secs(5)).await;
            file_name += "sleep.html"
        }
        uri if uri.starts_with(FS_ID) => {
            file_name += "fs.html";
            is_static_page = false;

            if uri == FS_ID {
                render_flags.fs_path = String::from("/");
            } else {
                render_flags.fs_path = uri[FS_ID.len()..].to_string();
            }
        }
        _ => {
            status_line = "HTTP/1.1 404 NOT FOUND".to_string();
            file_name += "404.html";
            is_valid_uri = false;
        }
    }

    // Attempt to read the response file and create response message
    let mut page_content = fs::read_to_string(&file_name)
        .await
        .map_err(|_| RequestError::FileNotFound(
            format!("File '{file_name}' not found").to_string(),
        ))?;

    // Render index page, if required
    if !is_static_page {
        renderer::render_index_page(&mut page_content, &render_flags, base_path.as_str())
            .await
            .map_err(|e| {
                match e {
                    RenderError::InvalidId(err_msg) => RequestError::RenderingError(err_msg),
                    _ => {
                        log::error!("File path not found");
                        RequestError::FilePathNotFound
                    }
            }
        })?;
    }

    // Create and send the response back upstream
    let response = create_http_response(status_line, page_content);
    stream
        .write_all(response.as_bytes())
        .await
        .map_err(|_| RequestError::StreamError("Unable to send request".to_string()))?;

    // Indicate result type to caller function
    if is_valid_uri {
        Ok(String::from("Response successful!"))
    } else {
        Err(RequestError::UnsupportedURI(uri.to_string()))
    }
}

fn create_http_response(status_line: String, page_content: String) -> String {
    let length = page_content.len();
    format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{page_content}")
}
