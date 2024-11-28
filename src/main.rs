use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use file_browser::renderer;

#[derive(Debug)]
enum ResultHandle{
    Ok(String),
    UnsupportedURI(String),
    InvalidRequest,
    InvalidMethod,
    StreamError(String),
    FileNotFound(String)
}


fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        let handle_result = handle_connection(stream);
        
        // If the handling of the request fails, deal with it
        if !handle_result.is_err(){
            match handle_result.unwrap(){
                ResultHandle::Ok(ok_msg) => println!("{ok_msg}"),
                _ => ()
            }
        }
        else {
            match handle_result.unwrap_err(){
                ResultHandle::UnsupportedURI(uri) => println!("URI '{uri}' is not currently supported"),
                ResultHandle::InvalidRequest => println!("Invalid request"),
                ResultHandle::InvalidMethod => println!("HTTP method not supported"),
                ResultHandle::StreamError(err) => println!("{err}"),
                ResultHandle::FileNotFound(err) => println!("{err}"),
                _ => (),
            }
        }

        
    }
}


fn handle_connection(mut stream: TcpStream) -> Result<ResultHandle, ResultHandle>{
    let pages_path = String::from("pages/");   
    let buf_reader = BufReader::new(&mut stream);

    // Get the http header tokens
    let request_line = buf_reader.lines().next().unwrap().unwrap();
    let request_tokens = request_line
        .split_whitespace()
        .collect::<Vec<&str>>();

    // Check for valid method
    match request_tokens[0]{
        "GET" => (),
        _ => return Err(ResultHandle::InvalidMethod)
    }

    // URI not present
    if request_tokens.len() < 2 {return Err(ResultHandle::InvalidRequest);}

    // Handle URI request
    let uri = request_tokens[1];
    let mut status_line: String = "HTTP/1.1 200 OK".to_string(); 
    let mut file_name: String = pages_path.clone();
    let mut is_valid_uri = true;
    let mut is_static_page = true;
    match uri{
        "/" | "/fs" => {
            file_name += "index.html";
            is_static_page = false;
        },
        "/flowers"  => file_name += "flowers.html",
        "/sleep"    => {
            thread::sleep(Duration::from_secs(5));
            file_name += "sleep.html"
        }
        _           => {
            status_line = "HTTP/1.1 404 NOT FOUND".to_string();
            file_name += "404.html";
            is_valid_uri = false;
        }
    }

    // Attempt to read the response file and create response message
    let page_content = fs::read_to_string(&file_name);
    let response;
    if page_content.is_err(){
        return Err(ResultHandle::FileNotFound(format!("File '{file_name}' not found").to_string()))
    }
    let mut page_content = page_content.unwrap();

    // Render index page, if required
    if !is_static_page { renderer::render_index_page(&mut page_content);}
    
    // Create and send the response back upstream
    response = create_http_response(status_line, page_content);
    stream.write_all(response.as_bytes()).map_err(|_| ResultHandle::StreamError("Unable to send request".to_string()))?;
    
    // Indicate result type to caller function
    if is_valid_uri{ Ok(ResultHandle::Ok(String::from("Response successful!")))}
    else{            Err(ResultHandle::UnsupportedURI(uri.to_string()))}
}


fn create_http_response(status_line: String, page_content: String) -> String{
    let length = page_content.len();
    format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{page_content}")
}

