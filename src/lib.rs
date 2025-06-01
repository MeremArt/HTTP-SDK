
// Re-export essential types from reqwest for convenience
pub use reqwest::{Method, StatusCode, Url};

// Public modules
pub mod client;
pub mod error;
pub mod middleware;

// Optional blocking client
#[cfg(feature = "blocking")]
pub mod blocking;

// Public exports
pub use client::{ClientConfig, HttpClient, RequestBuilderExt};
pub use error::{HttpError, Result};
pub use middleware::{
    AuthMiddleware, AuthType, HeaderMiddleware, LoggingMiddleware, 
    Middleware, RetryMiddleware
};

#[cfg(feature = "blocking")]
pub use blocking::{BlockingClientConfig, BlockingHttpClient, BlockingRequestBuilderExt};

// Utility functions and builders
pub mod utils;
pub use utils::*;

// Re-export common serialization traits
pub use serde::{Deserialize, Serialize};

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Create a new HTTP client with default settings
pub fn new_client() -> HttpClient {
    HttpClient::new()
}

/// Create a new HTTP client with a base URL
pub fn client_with_base_url<S: Into<String>>(base_url: S) -> HttpClient {
    HttpClient::with_base_url(base_url)
}

/// Create a new blocking HTTP client with default settings
#[cfg(feature = "blocking")]
pub fn new_blocking_client() -> BlockingHttpClient {
    BlockingHttpClient::new()
}

/// Create a new blocking HTTP client with a base URL
#[cfg(feature = "blocking")]
pub fn blocking_client_with_base_url<S: Into<String>>(base_url: S) -> BlockingHttpClient {
    BlockingHttpClient::with_base_url(base_url)
}

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::client::{ClientConfig, HttpClient, RequestBuilderExt};
    pub use crate::error::{HttpError, Result};
    pub use crate::middleware::{AuthMiddleware, AuthType, Middleware};
    pub use crate::{new_client, client_with_base_url};
    
    #[cfg(feature = "blocking")]
    pub use crate::blocking::{BlockingClientConfig, BlockingHttpClient, BlockingRequestBuilderExt};
    #[cfg(feature = "blocking")]
    pub use crate::{new_blocking_client, blocking_client_with_base_url};
    
    pub use reqwest::{Method, StatusCode};
    pub use serde::{Deserialize, Serialize};
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
    
    #[test]
    fn test_client_creation() {
        let client = new_client();
        assert_eq!(client.middleware_count(), 0);
    }
    
    #[test]
    fn test_client_with_base_url() {
        let client = client_with_base_url("https://api.example.com");
        assert_eq!(
            client.config().base_url,
            Some("https://api.example.com".to_string())
        );
    }
    
    #[cfg(feature = "blocking")]
    #[test]
    fn test_blocking_client_creation() {
        let client = new_blocking_client();
        assert!(client.config().timeout.is_some());
    }
}