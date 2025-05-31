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
//create chainable methods for ClientConfig
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


impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for HttpClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HttpClient")
            .field("config", &self.config)
            .field("middleware_count", &self.middlewares.len())
            .finish()
    }
}

impl HttpClient {
    /// Create a new HTTP client with default settings
    pub fn new() -> Self {
        let config = ClientConfig::default(); //creates a config with all the default values.
        let client = Self::build_reqwest_client(&config)?;
        
        Self {
            client,
            config,
            middlewares: Vec::new(),
        }
    }
    
    /// Create a new HTTP client with custom configuration
    pub fn with_config(config: ClientConfig) -> Result<Self> {
        
        let client = Self::build_reqwest_client(&config)?;
        
        Ok(Self {
            client,
            config,
            middlewares: Vec::new(),
        })
    }
    
    /// Create a new HTTP client with a base URL
    pub fn with_base_url<S: Into<String>>(base_url: S) -> Self {
        let config = ClientConfig::default().with_base_url(base_url);
        Self::with_config(config).unwrap()
    }
    
    /// Add middleware to the client
    pub fn with_middleware<M: Middleware + 'static>(mut self, middleware: M) -> Self {
        self.middlewares.push(Arc::new(middleware));
        self
    }
    
    /// Build the underlying reqwest client
    fn build_reqwest_client(config: &ClientConfig) -> Result<Client> {
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
    
    /// Execute a request with middleware processing
    async fn execute_request(&self, mut request: reqwest::Request) -> Result<Response> {
        // Process request through middleware
        for middleware in &self.middlewares {
            middleware.process_request(&mut request).await?;
        }
        
        let mut response = self.client.execute(request).await?;
        
        // Process response through middleware
        for middleware in &self.middlewares {
            middleware.process_response(&mut response).await?;
        }
        
        Ok(response)
    }
    
    /// Send a GET request
    pub async fn get(&self, url: &str) -> Result<Response> {
        let request = self.request(Method::GET, url)?.build()?;
        self.execute_request(request).await
    }
    
    /// Send a GET request and deserialize the response as JSON
    pub async fn get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let response = self.get(url).await?;
        self.process_json_response(response).await
    }
    
    /// Send a POST request
    pub async fn post(&self, url: &str) -> Result<Response> {
        let request = self.request(Method::POST, url)?.build()?;
        self.execute_request(request).await
    }
    
    /// Send a POST request with a JSON body
    pub async fn post_json<T: Serialize, R: DeserializeOwned>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<R> {
        let request = self.request(Method::POST, url)?.json(body).build()?;
        let response = self.execute_request(request).await?;
        self.process_json_response(response).await
    }
    
    /// Send a PUT request
    pub async fn put(&self, url: &str) -> Result<Response> {
        let request = self.request(Method::PUT, url)?.build()?;
        self.execute_request(request).await
    }
    
    /// Send a PUT request with a JSON body
    pub async fn put_json<T: Serialize, R: DeserializeOwned>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<R> {
        let request = self.request(Method::PUT, url)?.json(body).build()?;
        let response = self.execute_request(request).await?;
        self.process_json_response(response).await
    }
    
    /// Send a DELETE request
    pub async fn delete(&self, url: &str) -> Result<Response> {
        let request = self.request(Method::DELETE, url)?.build()?;
        self.execute_request(request).await
    }
    
    /// Send a DELETE request and deserialize the response as JSON
    pub async fn delete_json<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let response = self.delete(url).await?;
        self.process_json_response(response).await
    }
    
    /// Send a PATCH request
    pub async fn patch(&self, url: &str) -> Result<Response> {
        let request = self.request(Method::PATCH, url)?.build()?;
        self.execute_request(request).await
    }
    
    /// Send a PATCH request with a JSON body
    pub async fn patch_json<T: Serialize, R: DeserializeOwned>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<R> {
        let request = self.request(Method::PATCH, url)?.json(body).build()?;
        let response = self.execute_request(request).await?;
        self.process_json_response(response).await
    }
    
    /// Send a HEAD request
    pub async fn head(&self, url: &str) -> Result<Response> {
        let request = self.request(Method::HEAD, url)?.build()?;
        self.execute_request(request).await
    }
    
    /// Helper method to process a JSON response
    async fn process_json_response<T: DeserializeOwned>(&self, response: Response) -> Result<T> {
        let status = response.status();
        
        if status.is_success() {
            response.json::<T>().await.map_err(|e| {
                HttpError::SerializationError(format!("Failed to deserialize response: {}", e))
            })
        } else {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Could not read error body".to_string());
            Err(HttpError::ResponseError { status, body })
        }
    }
    
    /// Send a request with custom headers
    pub async fn request_with_headers(
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
        
        let request = builder.build()?;
        self.execute_request(request).await
    }
    
    /// Send a request with query parameters
    pub async fn request_with_query<T: Serialize>(
        &self,
        method: Method,
        url: &str,
        params: &T,
    ) -> Result<Response> {
        let request = self.request(method, url)?.query(params).build()?;
        self.execute_request(request).await
    }
    
    /// Get client configuration
    pub fn config(&self) -> &ClientConfig {
        &self.config
    }
    
    /// Get middleware count
    pub fn middleware_count(&self) -> usize {
        self.middlewares.len()
    }
}

/// Extension trait for RequestBuilder to provide more fluent API
pub trait RequestBuilderExt {
    fn with_query<T: Serialize>(self, params: &T) -> RequestBuilder;
    fn with_header<K, V>(self, key: K, value: V) -> RequestBuilder
    where
        K: TryInto<HeaderName>,
        V: TryInto<HeaderValue>;
}

impl RequestBuilderExt for RequestBuilder {
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
    fn test_client_config_creation() {
        let config = ClientConfig::new()
            .with_base_url("https://api.example.com")
            .with_timeout(Duration::from_secs(60));
        
        assert_eq!(config.base_url, Some("https://api.example.com".to_string()));
        assert_eq!(config.timeout, Some(Duration::from_secs(60)));
    }
    
    #[test]
    fn test_client_creation() {
        let client = HttpClient::new();
        assert_eq!(client.middleware_count(), 0);
    }
    
    #[test]
    fn test_url_building() {
        let client = HttpClient::with_base_url("https://api.example.com");
        
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