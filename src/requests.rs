#[derive(Debug)]
pub enum RequestResult {
    FileNotFound(String),
    FilePathNotFound,
    InvalidMethod,
    InvalidRequest,
    Ok(String),
    RenderingError(String),
    StreamError(String),
    UnsupportedURI(String),
}
