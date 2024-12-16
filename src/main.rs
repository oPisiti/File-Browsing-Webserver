use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use file_browser::{renderer, RequestResult};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        let handle_result = handle_connection(stream);
        
        // If the handling of the request fails, deal with it
        if !handle_result.is_err(){
            match handle_result.unwrap(){
                RequestResult::Ok(ok_msg) => println!("{ok_msg}"),
                _ => ()
            }
        }
        else {
            match handle_result.unwrap_err(){
                RequestResult::UnsupportedURI(uri) => println!("URI '{uri}' is not currently supported"),
                RequestResult::InvalidRequest => println!("Invalid request"),
                RequestResult::InvalidMethod => println!("HTTP method not supported"),
                RequestResult::StreamError(err) => println!("{err}"),
                RequestResult::FileNotFound(err) => println!("{err}"),
                RequestResult::FilePathNotFound => println!("Path specified in the url was not found"),
                _ => (),
            }
        }

        
    }
}


fn handle_connection(mut stream: TcpStream) -> Result<RequestResult, RequestResult>{
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
        _ => return Err(RequestResult::InvalidMethod)
    }

    // URI not present
    if request_tokens.len() < 2 {return Err(RequestResult::InvalidRequest);}

    // Handle URI request
    let uri = request_tokens[1];
    let mut status_line: String = "HTTP/1.1 200 OK".to_string(); 
    let mut file_name: String = pages_path.clone();
    let mut is_valid_uri = true;
    let mut is_static_page = true;
    let mut render_flags = renderer::RenderFlags::default();
    match uri{
        "/"         => file_name += "puppy.html",
        "/flowers"  => file_name += "flowers.html",
        "/sleep"    => {
            thread::sleep(Duration::from_secs(5));
            file_name += "sleep.html"
        }
        uri if uri.starts_with("/fs") => {
            file_name += "index.html";
            is_static_page = false;
            
            if uri == "/fs" {render_flags.fs_path = String::from("/");}
            else            {render_flags.fs_path = uri["/fs".len()..].to_string();}
        },
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
        return Err(RequestResult::FileNotFound(format!("File '{file_name}' not found").to_string()))
    }
    let mut page_content = page_content.unwrap();

    // Render index page, if required
    if !is_static_page { 
        renderer::render_index_page(&mut page_content, &render_flags)
            .map_err(|_| RequestResult::FilePathNotFound)?;
    }
    
    // Create and send the response back upstream
    response = create_http_response(status_line, page_content);
    stream.write_all(response.as_bytes()).map_err(|_| RequestResult::StreamError("Unable to send request".to_string()))?;
    
    // Indicate result type to caller function
    if is_valid_uri{ Ok(RequestResult::Ok(String::from("Response successful!")))}
    else{            Err(RequestResult::UnsupportedURI(uri.to_string()))}
}


fn create_http_response(status_line: String, page_content: String) -> String{
    let length = page_content.len();
    format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{page_content}")
}

