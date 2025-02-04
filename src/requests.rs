#[derive(Debug)]
pub enum RequestError {
    FileNotFound(String),
    FilePathNotFound,
    InvalidMethod,
    InvalidRequest,
    RenderingError(String),
    StreamError(String),
    UnsupportedURI(String),
}
