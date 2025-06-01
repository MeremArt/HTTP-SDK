// src/middleware.rs
use crate::error::{HttpError, Result};
use reqwest::{Request, Response};
use std::fmt;

/// Trait for implementing request/response middleware
#[async_trait::async_trait]
pub trait Middleware: Send + Sync + fmt::Debug {
    /// Process the request before it's sent
    async fn process_request(&self, request: &mut Request) -> Result<()>;
    
    /// Process the response after it's received
    async fn process_response(&self, response: &mut Response) -> Result<()>;
    
    /// Get the name of this middleware for debugging
    fn name(&self) -> &'static str;
}

/// Middleware for adding authentication headers
#[derive(Debug, Clone)]
pub struct AuthMiddleware {
    pub token: String,
    pub auth_type: AuthType,
}

#[derive(Debug, Clone)]
pub enum AuthType {
    Bearer,
    Basic,
    ApiKey(String), // header name
}

impl AuthMiddleware {
    pub fn bearer(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            auth_type: AuthType::Bearer,
        }
    }
    
    pub fn basic(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            auth_type: AuthType::Basic,
        }
    }
    
    pub fn api_key(header_name: impl Into<String>, token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            auth_type: AuthType::ApiKey(header_name.into()),
        }
    }
}

#[async_trait::async_trait]
impl Middleware for AuthMiddleware {
    async fn process_request(&self, request: &mut Request) -> Result<()> {
        let headers = request.headers_mut();
        
        match &self.auth_type {
            AuthType::Bearer => {
                let value = format!("Bearer {}", self.token);
                headers.insert(
                    reqwest::header::AUTHORIZATION,
                    value.parse().map_err(|_| {
                        HttpError::MiddlewareError("Invalid bearer token".to_string())
                    })?,
                );
            }
            AuthType::Basic => {
                let value = format!("Basic {}", self.token);
                headers.insert(
                    reqwest::header::AUTHORIZATION,
                    value.parse().map_err(|_| {
                        HttpError::MiddlewareError("Invalid basic auth token".to_string())
                    })?,
                );
            }
            AuthType::ApiKey(header_name) => {
                let header_name = reqwest::header::HeaderName::from_bytes(header_name.as_bytes())
                    .map_err(|_| {
                        HttpError::MiddlewareError(format!("Invalid header name: {}", header_name))
                    })?;
                
                headers.insert(
                    header_name,
                    self.token.parse().map_err(|_| {
                        HttpError::MiddlewareError("Invalid API key".to_string())
                    })?,
                );
            }
        }
        
        Ok(())
    }
    
    async fn process_response(&self, _response: &mut Response) -> Result<()> {
        // Auth middleware doesn't need to process responses
        Ok(())
    }
    
    fn name(&self) -> &'static str {
        "AuthMiddleware"
    }
}

/// Middleware for adding custom headers to requests
#[derive(Debug, Clone)]
pub struct HeaderMiddleware {
    pub headers: std::collections::HashMap<String, String>,
}

impl HeaderMiddleware {
    pub fn new() -> Self {
        Self {
            headers: std::collections::HashMap::new(),
        }
    }
    
    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }
}

impl Default for HeaderMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Middleware for HeaderMiddleware {
    async fn process_request(&self, request: &mut Request) -> Result<()> {
        let headers = request.headers_mut();
        
        for (name, value) in &self.headers {
            let header_name = reqwest::header::HeaderName::from_bytes(name.as_bytes())
                .map_err(|_| {
                    HttpError::MiddlewareError(format!("Invalid header name: {}", name))
                })?;
            
            let header_value = reqwest::header::HeaderValue::from_str(value)
                .map_err(|_| {
                    HttpError::MiddlewareError(format!("Invalid header value: {}", value))
                })?;
            
            headers.insert(header_name, header_value);
        }
        
        Ok(())
    }
    
    async fn process_response(&self, _response: &mut Response) -> Result<()> {
        Ok(())
    }
    
    fn name(&self) -> &'static str {
        "HeaderMiddleware"
    }
}

/// Middleware for logging requests and responses
#[derive(Debug, Clone)]
pub struct LoggingMiddleware {
    pub log_requests: bool,
    pub log_responses: bool,
}

impl LoggingMiddleware {
    pub fn new() -> Self {
        Self {
            log_requests: true,
            log_responses: true,
        }
    }
    
    pub fn requests_only() -> Self {
        Self {
            log_requests: true,
            log_responses: false,
        }
    }
    
    pub fn responses_only() -> Self {
        Self {
            log_requests: false,
            log_responses: true,
        }
    }
}

impl Default for LoggingMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Middleware for LoggingMiddleware {
    async fn process_request(&self, request: &mut Request) -> Result<()> {
        if self.log_requests {
            log::info!("HTTP Request: {} {}", request.method(), request.url());
            
            if log::log_enabled!(log::Level::Debug) {
                log::debug!("Request headers: {:?}", request.headers());
            }
        }
        
        Ok(())
    }
    
    async fn process_response(&self, response: &mut Response) -> Result<()> {
        if self.log_responses {
            log::info!("HTTP Response: {} {}", response.status(), response.url());
            
            if log::log_enabled!(log::Level::Debug) {
                log::debug!("Response headers: {:?}", response.headers());
            }
        }
        
        Ok(())
    }
    
    fn name(&self) -> &'static str {
        "LoggingMiddleware"
    }
}

/// Middleware for retrying failed requests
#[derive(Debug, Clone)]
pub struct RetryMiddleware {
    pub max_retries: u32,
    pub retry_delay_ms: u64,
}

impl RetryMiddleware {
    pub fn new(max_retries: u32) -> Self {
        Self {
            max_retries,
            retry_delay_ms: 1000,
        }
    }
    
    pub fn with_delay(mut self, delay_ms: u64) -> Self {
        self.retry_delay_ms = delay_ms;
        self
    }
}

#[async_trait::async_trait]
impl Middleware for RetryMiddleware {
    async fn process_request(&self, _request: &mut Request) -> Result<()> {
        // Retry logic is handled at the client level
        Ok(())
    }
    
    async fn process_response(&self, _response: &mut Response) -> Result<()> {
        // Retry logic is handled at the client level
        Ok(())
    }
    
    fn name(&self) -> &'static str {
        "RetryMiddleware"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_auth_middleware_creation() {
        let middleware = AuthMiddleware::bearer("test-token");
        assert_eq!(middleware.token, "test-token");
        assert!(matches!(middleware.auth_type, AuthType::Bearer));
    }
    
    #[test]
    fn test_header_middleware_creation() {
        let middleware = HeaderMiddleware::new()
            .with_header("X-Custom", "value")
            .with_header("X-Another", "another-value");
        
        assert_eq!(middleware.headers.len(), 2);
        assert_eq!(middleware.headers.get("X-Custom"), Some(&"value".to_string()));
    }
}