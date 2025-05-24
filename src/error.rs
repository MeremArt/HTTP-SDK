// src/error.rs
use reqwest::StatusCode;
use thiserror::Error;

/// Custom error type for the HTTP client SDK
#[derive(Error, Debug)]
pub enum HttpError {
    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("HTTP error {status}: {body}")]
    ResponseError { 
        status: StatusCode, 
        body: String 
    },
    
    #[error("Header error: {0}")]
    HeaderError(String),
    
    #[error("URL error: {0}")]
    UrlError(String),
    
    #[error("Timeout error")]
    TimeoutError,
    
    #[error("JSON error: {0}")]
    JsonError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Middleware error: {0}")]
    MiddlewareError(String),
}

/// Result type alias to simplify return types
pub type Result<T> = std::result::Result<T, HttpError>;

impl From<serde_json::Error> for HttpError {
    fn from(err: serde_json::Error) -> Self {
        HttpError::JsonError(err.to_string())
    }
}

impl From<url::ParseError> for HttpError {
    fn from(err: url::ParseError) -> Self {
        HttpError::UrlError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_display() {
        let error = HttpError::HeaderError("Invalid header".to_string());
        assert_eq!(error.to_string(), "Header error: Invalid header");
    }
    
    #[test]
    fn test_response_error() {
        let error = HttpError::ResponseError {
            status: StatusCode::NOT_FOUND,
            body: "Not found".to_string(),
        };
        assert_eq!(error.to_string(), "HTTP error 404 Not Found: Not found");
    }
}