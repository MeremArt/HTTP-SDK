

use crate::error::{HttpError, Result};
use reqwest::blocking::{Client, RequestBuilder, Response};
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Method, StatusCode,
};
use serde::{de::DeserializeOwned, Serialize};
use std::{collections::HashMap, fmt, time::Duration};

/// Configuration for the blocking HTTP client
#[derive(Debug, Clone)]
pub struct BlockingClientConfig {
    pub base_url: Option<String>,
    pub timeout: Option<Duration>,
    pub default_headers: HeaderMap,
    pub follow_redirects: bool,
    pub max_redirects: u32,
    pub connect_timeout: Option<Duration>,
    pub pool_idle_timeout: Option<Duration>,
    pub pool_max_idle_per_host: Option<usize>,
}

impl Default for BlockingClientConfig {
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

impl BlockingClientConfig {
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

/// Blocking HTTP client struct
pub struct BlockingHttpClient {
    client: Client,
    config: BlockingClientConfig,
}

impl fmt::Debug for BlockingHttpClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BlockingHttpClient")
            .field("config", &self.config)
            .finish()
    }
}

impl Default for BlockingHttpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl BlockingHttpClient {
    /// Create a new blocking HTTP client with default settings
    pub fn new() -> Self {
        let config = BlockingClientConfig::default();
        let client = Self::build_reqwest_client(&config).unwrap();
        
        Self { client, config }
    }
    
    /// Create a new blocking HTTP client with custom configuration
    pub fn with_config(config: BlockingClientConfig) -> Result<Self> {
        let client = Self::build_reqwest_client(&config)?;
        
        Ok(Self { client, config })
    }
    
    /// Create a new blocking HTTP client with a base URL
    pub fn with_base_url<S: Into<String>>(base_url: S) -> Self {
        let config = BlockingClientConfig::default().with_base_url(base_url);
        Self::with_config(config).unwrap()
    }
    
    /// Build the underlying reqwest blocking client
    fn build_reqwest_client(config: &BlockingClientConfig) -> Result<Client> {
        let mut builder = Client::builder();
        
        if let Some(timeout) = config.timeout {
            builder = builder.timeout(timeout);
        }
        
        if let Some(connect_timeout) = config.connect_timeout {
            builder = builder.connect_timeout(connect_timeout);
        }
        
        if let Some(pool_idle_timeout) = config.pool_idle_timeout {
            builder = builder.pool_idle_timeout(pool_idle_timeout);
        }
        
        if let Some(pool_max_idle_per_host) = config.pool_max_idle_per_host {
            builder = builder.pool_max_idle_per_host(pool_max_idle_per_host);
        }
        
        builder = builder
            .redirect(if config.follow_redirects {
                reqwest::redirect::Policy::limited(config.max_redirects as usize)
            } else {
                reqwest::redirect::Policy::none()
            })
            .default_headers(config.default_headers.clone());
        
        builder.build().map_err(HttpError::from)
    }
    
    /// Build the complete URL with the base URL
    fn build_url(&self, url: &str) -> Result<String> {
        match &self.config.base_url {
            Some(base) if !url.starts_with("http") => {
                let mut full_url = base.clone();
                if !base.ends_with('/') && !url.starts_with('/') {
                    full_url.push('/');
                } else if base.ends_with('/') && url.starts_with('/') {
                    full_url.pop();
                }
                full_url.push_str(url);
                Ok(full_url)
            }
            _ => Ok(url.to_string()),
        }
    }
    
    /// Create a request builder with common settings
    pub fn request(&self, method: Method, url: &str) -> Result<RequestBuilder> {
        let full_url = self.build_url(url)?;
        let builder = self.client.request(method, &full_url);
        Ok(builder)
    }
    
    /// Send a GET request
    pub fn get(&self, url: &str) -> Result<Response> {
        self.request(Method::GET, url)?
            .send()
            .map_err(HttpError::from)
    }
    
    /// Send a GET request and deserialize the response as JSON
    pub fn get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let response = self.get(url)?;
        self.process_json_response(response)
    }
    
    /// Send a POST request
    pub fn post(&self, url: &str) -> Result<Response> {
        self.request(Method::POST, url)?
            .send()
            .map_err(HttpError::from)
    }
    
    /// Send a POST request with a JSON body
    pub fn post_json<T: Serialize, R: DeserializeOwned>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<R> {
        let response = self.request(Method::POST, url)?
            .json(body)
            .send()
            .map_err(HttpError::from)?;
        
        self.process_json_response(response)
    }
    
    /// Send a PUT request
    pub fn put(&self, url: &str) -> Result<Response> {
        self.request(Method::PUT, url)?
            .send()
            .map_err(HttpError::from)
    }
    
    /// Send a PUT request with a JSON body
    pub fn put_json<T: Serialize, R: DeserializeOwned>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<R> {
        let response = self.request(Method::PUT, url)?
            .json(body)
            .send()
            .map_err(HttpError::from)?;
        
        self.process_json_response(response)
    }
    
    /// Send a DELETE request
    pub fn delete(&self, url: &str) -> Result<Response> {
        self.request(Method::DELETE, url)?
            .send()
            .map_err(HttpError::from)
    }
    
    /// Send a DELETE request and deserialize the response as JSON
    pub fn delete_json<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let response = self.delete(url)?;
        self.process_json_response(response)
    }
    
    /// Send a PATCH request
    pub fn patch(&self, url: &str) -> Result<Response> {
        self.request(Method::PATCH, url)?
            .send()
            .map_err(HttpError::from)
    }
    
    /// Send a PATCH request with a JSON body
    pub fn patch_json<T: Serialize, R: DeserializeOwned>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<R> {
        let response = self.request(Method::PATCH, url)?
            .json(body)
            .send()
            .map_err(HttpError::from)?;
        
        self.process_json_response(response)
    }
    
    /// Send a HEAD request
    pub fn head(&self, url: &str) -> Result<Response> {
        self.request(Method::HEAD, url)?
            .send()
            .map_err(HttpError::from)
    }
    
    /// Helper method to process a JSON response
    fn process_json_response<T: DeserializeOwned>(&self, response: Response) -> Result<T> {
        let status = response.status();
        
        if status.is_success() {
            response.json::<T>().map_err(|e| {
                HttpError::SerializationError(format!("Failed to deserialize response: {}", e))
            })
        } else {
            let body = response
                .text()
                .unwrap_or_else(|_| "Could not read error body".to_string());
            Err(HttpError::ResponseError { status, body })
        }
    }
    
    /// Send a request with custom headers
    pub fn request_with_headers(
        &self,
        method: Method,
        url: &str,
        headers: HashMap<String, String>,
    ) -> Result<Response> {
        let mut builder = self.request(method, url)?;
        
        for (key, value) in headers {
            let header_name = HeaderName::from_bytes(key.as_bytes())
                .map_err(|_| HttpError::HeaderError(format!("Invalid header name: {}", key)))?;
            
            let header_value = HeaderValue::from_str(&value)
                .map_err(|_| HttpError::HeaderError(format!("Invalid header value: {}", value)))?;
            
            builder = builder.header(header_name, header_value);
        }
        
        builder.send().map_err(HttpError::from)
    }
    
    /// Send a request with query parameters
    pub fn request_with_query<T: Serialize>(
        &self,
        method: Method,
        url: &str,
        params: &T,
    ) -> Result<Response> {
        self.request(method, url)?
            .query(params)
            .send()
            .map_err(HttpError::from)
    }
    
    /// Get client configuration
    pub fn config(&self) -> &BlockingClientConfig {
        &self.config
    }
    
    /// Execute a form request
    pub fn post_form<T: Serialize, R: DeserializeOwned>(
        &self,
        url: &str,
        form: &T,
    ) -> Result<R> {
        let response = self.request(Method::POST, url)?
            .form(form)
            .send()
            .map_err(HttpError::from)?;
        
        self.process_json_response(response)
    }
    
    /// Execute a multipart form request
    pub fn post_multipart<R: DeserializeOwned>(
        &self,
        url: &str,
        form: reqwest::blocking::multipart::Form,
    ) -> Result<R> {
        let response = self.request(Method::POST, url)?
            .multipart(form)
            .send()
            .map_err(HttpError::from)?;
        
        self.process_json_response(response)
    }
    
    /// Download a file to bytes
    pub fn download_bytes(&self, url: &str) -> Result<Vec<u8>> {
        let response = self.get(url)?;
        let status = response.status();
        
        if status.is_success() {
            response.bytes()
                .map(|bytes| bytes.to_vec())
                .map_err(HttpError::from)
        } else {
            let body = response
                .text()
                .unwrap_or_else(|_| "Could not read error body".to_string());
            Err(HttpError::ResponseError { status, body })
        }
    }
    
    /// Stream download to a writer
    pub fn download_to_writer<W: std::io::Write>(
        &self,
        url: &str,
        mut writer: W,
    ) -> Result<u64> {
        let mut response = self.get(url)?;
        let status = response.status();
        
        if status.is_success() {
            std::io::copy(&mut response, &mut writer)
            .map_err(|e| HttpError::RequestError(reqwest::Error::new(reqwest::ErrorKind::Body, e)))


        } else {
            let body = response
                .text()
                .unwrap_or_else(|_| "Could not read error body".to_string());
            Err(HttpError::ResponseError { status, body })
        }
    }
}

/// Extension trait for blocking RequestBuilder
pub trait BlockingRequestBuilderExt {
    fn with_query<T: Serialize>(self, params: &T) -> RequestBuilder;
    fn with_header<K, V>(self, key: K, value: V) -> RequestBuilder
    where
        K: TryInto<HeaderName>,
        V: TryInto<HeaderValue>;
}

impl BlockingRequestBuilderExt for RequestBuilder {
    fn with_query<T: Serialize>(self, params: &T) -> RequestBuilder {
        self.query(params)
    }
    
    fn with_header<K, V>(self, key: K, value: V) -> RequestBuilder
    where
        K: TryInto<HeaderName>,
        V: TryInto<HeaderValue>,
    {
        if let (Ok(name), Ok(value)) = (key.try_into(), value.try_into()) {
            self.header(name, value)
        } else {
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_blocking_client_config_creation() {
        let config = BlockingClientConfig::new()
            .with_base_url("https://api.example.com")
            .with_timeout(Duration::from_secs(60));
        
        assert_eq!(config.base_url, Some("https://api.example.com".to_string()));
        assert_eq!(config.timeout, Some(Duration::from_secs(60)));
    }
    
    #[test]
    fn test_blocking_client_creation() {
        let client = BlockingHttpClient::new();
        assert!(client.config.timeout.is_some());
    }
    
    #[test]
    fn test_blocking_url_building() {
        let client = BlockingHttpClient::with_base_url("https://api.example.com");
        
        assert_eq!(
            client.build_url("/users").unwrap(),
            "https://api.example.com/users"
        );
        
        assert_eq!(
            client.build_url("users").unwrap(),
            "https://api.example.com/users"
        );
        
        assert_eq!(
            client.build_url("https://other.com/test").unwrap(),
            "https://other.com/test"
        );
    }
}