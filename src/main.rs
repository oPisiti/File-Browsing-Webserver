use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

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

        // println!("{:#?}", handle_connection(stream));  
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
    let file_name: &str;
    let mut valid_uri = true;
    match uri{
        "/"        => file_name = "puppy.html",
        "/flowers" => file_name = "flowers.html",
        _ => {
            status_line = "HTTP/1.1 404 NOT FOUND".to_string();
            file_name = "404.html";
            valid_uri = false;
        }
    }

    // Attempt to read the response file
    let page_content = fs::read_to_string(file_name);
    let response;
    if page_content.is_err(){
        return Err(ResultHandle::FileNotFound("File '{file_name}' not found".to_string()))
    }
    else{
        response = create_http_response(status_line, page_content.unwrap());
    }

    // Create and send it back 
    stream.write_all(response.as_bytes()).map_err(|_| ResultHandle::StreamError("Unable to send request".to_string()))?;
    
    if valid_uri {
        Ok(ResultHandle::Ok(String::from("Response successful!")))
    }
    else{
        Err(ResultHandle::UnsupportedURI(uri.to_string()))
    }
}

fn create_http_response(status_line: String, page_content: String) -> String{
    let length = page_content.len();
    format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{page_content}")
}