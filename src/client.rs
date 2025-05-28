use crate::error::{HttpError, Result};
use crate::middleware::Middleware;
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Client, Method, RequestBuilder, Response, StatusCode,
};
use serde::{de::DeserializeOwned, Serialize}; //with full type ownership (no borrowing).
use std::{collections::HashMap, fmt, sync::Arc, time::Duration};

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub base_url: Option<String>,
    pub timeout: Option<Duration>,
    pub default_headers: HeaderMap,
    pub follow_redirects: bool,
    pub max_redirects: u32,
    pub connect_timeout: Option<Duration>,
    pub pool_idle_timeout: Option<Duration>,
    pub pool_max_idle_per_host: Option<usize>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            base_url: None,
            timeout: Some(Duration::from_secs(30)),
            default_headers: HeaderMap::new(),
            follow_redirects: true,
            max_redirects: 10,
            connect_timeout: Some(Duration::from_secs(10)),
            pool_idle_timeout: Some(Duration::from_secs(90)),
            pool_max_idle_per_host: Some(10),
        }
    }
}

impl ClientConfig {
    /// Create a new client configuration
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set the base URL for all requests
    pub fn with_base_url<S: Into<String>>(mut self, base_url: S) -> Self {
        self.base_url = Some(base_url.into());
        self
    }
    
    /// Set the request timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
    
    /// Add a default header
    pub fn with_default_header<K, V>(mut self, key: K, value: V) -> Result<Self>
    where
        K: TryInto<HeaderName>,
        K::Error: fmt::Display,
        V: TryInto<HeaderValue>,
        V::Error: fmt::Display,
    {
        let header_name = key.try_into()
            .map_err(|e| HttpError::HeaderError(e.to_string()))?;
        
        let header_value = value.try_into()
            .map_err(|e| HttpError::HeaderError(e.to_string()))?;
        
        self.default_headers.insert(header_name, header_value);
        Ok(self)
    }
    
    /// Set JSON content type headers
    pub fn with_json_headers(self) -> Result<Self> {
        self.with_default_header("Content-Type", "application/json")?
            .with_default_header("Accept", "application/json")
    }
    
    /// Configure redirect behavior
    pub fn with_redirects(mut self, follow: bool, max_redirects: u32) -> Self {
        self.follow_redirects = follow;
        self.max_redirects = max_redirects;
        self
    }
    
    /// Set connection timeout
    pub fn with_connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = Some(timeout);
        self
    }
}

pub struct HttpClient {
    client: Client,
    config: ClientConfig,
    middlewares: Vec<Arc<dyn Middleware>>,
}
