// src/utils.rs
// Utility functions and helper types for the HTTP client

use crate::error::{HttpError, Result};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::Serialize;
use std::collections::HashMap;
use std::fmt;

/// Builder for creating HeaderMaps easily
#[derive(Debug, Clone, Default)]
pub struct HeaderBuilder {
    headers: HeaderMap,
}

impl HeaderBuilder {
    /// Create a new header builder
    pub fn new() -> Self {
        Self {
            headers: HeaderMap::new(),
        }
    }
    
    /// Add a header to the builder
    pub fn header<K, V>(mut self, key: K, value: V) -> Result<Self>
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
        
        self.headers.insert(header_name, header_value);
        Ok(self)
    }
    
    /// Add multiple headers from a HashMap
    pub fn headers(mut self, headers: HashMap<String, String>) -> Result<Self> {
        for (key, value) in headers {
            self = self.header(key, value)?;
        }
        Ok(self)
    }
    
    /// Add JSON content type headers
    pub fn json_headers(self) -> Result<Self> {
        self.header("Content-Type", "application/json")?
            .header("Accept", "application/json")
    }
    
    /// Add authorization bearer token
    pub fn bearer_auth<T: fmt::Display>(self, token: T) -> Result<Self> {
        self.header("Authorization", format!("Bearer {}", token))
    }
    
    /// Add authorization basic auth
    pub fn basic_auth<T: fmt::Display>(self, token: T) -> Result<Self> {
        self.header("Authorization", format!("Basic {}", token))
    }
    
    /// Add API key header
    pub fn api_key<K: fmt::Display, V: fmt::Display>(self, header_name: K, api_key: V) -> Result<Self> {
        self.header(header_name.to_string(), api_key.to_string())
    }
    
    /// Add user agent header
    pub fn user_agent<T: fmt::Display>(self, user_agent: T) -> Result<Self> {
        self.header("User-Agent", user_agent.to_string())
    }
    
    /// Build the final HeaderMap
    pub fn build(self) -> HeaderMap {
        self.headers
    }
}

/// Builder for creating query parameters
#[derive(Debug, Clone, Default)]
pub struct QueryBuilder {
    params: Vec<(String, String)>,
}

impl QueryBuilder {
    /// Create a new query builder
    pub fn new() -> Self {
        Self {
            params: Vec::new(),
        }
    }
    
    /// Add a query parameter
    pub fn param<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.params.push((key.into(), value.into()));
        self
    }
    
    /// Add multiple query parameters from a HashMap
    pub fn params(mut self, params: HashMap<String, String>) -> Self {
        for (key, value) in params {
            self.params.push((key, value));
        }
        self
    }
    
    /// Add a parameter only if the value is Some
    pub fn optional_param<K, V>(self, key: K, value: Option<V>) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        match value {
            Some(v) => self.param(key, v),
            None => self,
        }
    }
    
    /// Build the final query parameters as a vector of tuples
    pub fn build(self) -> Vec<(String, String)> {
        self.params
    }
    
    /// Build as a URL query string
    pub fn build_query_string(self) -> String {
        if self.params.is_empty() {
            return String::new();
        }
        
        let query: Vec<String> = self.params
            .into_iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(&k), urlencoding::encode(&v)))
            .collect();
        
        format!("?{}", query.join("&"))
    }
}

/// URL builder for constructing URLs with path segments and query parameters
#[derive(Debug, Clone)]
pub struct UrlBuilder {
    base_url: String,
    path_segments: Vec<String>,
    query_params: Vec<(String, String)>,
}

impl UrlBuilder {
    /// Create a new URL builder with a base URL
    pub fn new<S: Into<String>>(base_url: S) -> Self {
        Self {
            base_url: base_url.into(),
            path_segments: Vec::new(),
            query_params: Vec::new(),
        }
    }
    
    /// Add a path segment
    pub fn path<S: Into<String>>(mut self, segment: S) -> Self {
        self.path_segments.push(segment.into());
        self
    }
    
    /// Add multiple path segments
    pub fn paths<I, S>(mut self, segments: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for segment in segments {
            self.path_segments.push(segment.into());
        }
        self
    }
    
    /// Add a query parameter
    pub fn query<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.query_params.push((key.into(), value.into()));
        self
    }
    
    /// Add multiple query parameters
    pub fn queries<I, K, V>(mut self, params: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        for (key, value) in params {
            self.query_params.push((key.into(), value.into()));
        }
        self
    }
    
    /// Build the final URL
    pub fn build(self) -> String {
        let mut url = self.base_url;
        
        // Ensure base URL doesn't end with '/'
        if url.ends_with('/') {
            url.pop();
        }
        
        // Add path segments
        for segment in self.path_segments {
            url.push('/');
            url.push_str(&urlencoding::encode(&segment));
        }
        
        // Add query parameters
        if !self.query_params.is_empty() {
            url.push('?');
            let query_string: Vec<String> = self.query_params
                .into_iter()
                .map(|(k, v)| format!("{}={}", urlencoding::encode(&k), urlencoding::encode(&v)))
                .collect();
            url.push_str(&query_string.join("&"));
        }
        
        url
    }
}

/// Helper function to create a HeaderBuilder
pub fn headers() -> HeaderBuilder {
    HeaderBuilder::new()
}

/// Helper function to create a QueryBuilder
pub fn query() -> QueryBuilder {
    QueryBuilder::new()
}

/// Helper function to create a UrlBuilder
pub fn url<S: Into<String>>(base_url: S) -> UrlBuilder {
    UrlBuilder::new(base_url)
}

/// Convert a serializable struct to query parameters
pub fn to_query_params<T: Serialize>(params: &T) -> Result<Vec<(String, String)>> {
    let value = serde_json::to_value(params)
        .map_err(|e| HttpError::SerializationError(e.to_string()))?;
    
    let mut query_params = Vec::new();
    
    if let serde_json::Value::Object(map) = value {
        for (key, value) in map {
            match value {
                serde_json::Value::String(s) => {
                    query_params.push((key, s));
                }
                serde_json::Value::Number(n) => {
                    query_params.push((key, n.to_string()));
                }
                serde_json::Value::Bool(b) => {
                    query_params.push((key, b.to_string()));
                }
                serde_json::Value::Array(arr) => {
                    for item in arr {
                        if let Ok(s) = serde_json::to_string(&item) {
                            query_params.push((key.clone(), s.trim_matches('"').to_string()));
                        }
                    }
                }
                _ => {
                    // Skip null and complex objects
                }
            }
        }
    }
    
    Ok(query_params)
}

/// Encode a value for use in URLs
pub fn url_encode<T: fmt::Display>(value: T) -> String {
    urlencoding::encode(&value.to_string()).into_owned()
}

/// Format a duration as a human-readable string
pub fn format_duration(duration: std::time::Duration) -> String {
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();
    
    if secs > 0 {
        format!("{}.{}s", secs, millis / 100)
    } else {
        format!("{}ms", millis)
    }
}

/// Validate that a URL is well-formed
pub fn validate_url(url: &str) -> Result<()> {
    reqwest::Url::parse(url)
        .map_err(|e| HttpError::UrlError(format!("Invalid URL '{}': {}", url, e)))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;
    
    #[test]
    fn test_header_builder() {
        let headers = HeaderBuilder::new()
            .header("Content-Type", "application/json").unwrap()
            .header("User-Agent", "test-client").unwrap()
            .build();
        
        assert_eq!(headers.len(), 2);
        assert_eq!(headers.get("content-type").unwrap(), "application/json");
        assert_eq!(headers.get("user-agent").unwrap(), "test-client");
    }
    
    #[test]
    fn test_header_builder_json() {
        let headers = HeaderBuilder::new()
            .json_headers().unwrap()
            .build();
        
        assert_eq!(headers.len(), 2);
        assert_eq!(headers.get("content-type").unwrap(), "application/json");
        assert_eq!(headers.get("accept").unwrap(), "application/json");
    }
    
    #[test]
    fn test_header_builder_auth() {
        let headers = HeaderBuilder::new()
            .bearer_auth("token123").unwrap()
            .build();
        
        assert_eq!(headers.len(), 1);
        assert_eq!(headers.get("authorization").unwrap(), "Bearer token123");
    }
    
    #[test]
    fn test_query_builder() {
        let params = QueryBuilder::new()
            .param("name", "john")
            .param("age", "30")
            .build();
        
        assert_eq!(params.len(), 2);
        assert_eq!(params[0], ("name".to_string(), "john".to_string()));
        assert_eq!(params[1], ("age".to_string(), "30".to_string()));
    }
    
    #[test]
    fn test_query_builder_optional() {
        let params = QueryBuilder::new()
            .param("required", "value")
            .optional_param("optional", Some("present"))
            .optional_param("missing", None::<String>)
            .build();
        
        assert_eq!(params.len(), 2);
        assert_eq!(params[0], ("required".to_string(), "value".to_string()));
        assert_eq!(params[1], ("optional".to_string(), "present".to_string()));
    }
    
    #[test]
    fn test_query_string_building() {
        let query_string = QueryBuilder::new()
            .param("name", "john doe")
            .param("city", "new york")
            .build_query_string();
        
        assert!(query_string.contains("name=john%20doe"));
        assert!(query_string.contains("city=new%20york"));
        assert!(query_string.starts_with('?'));
    }
    
    #[test]
    fn test_url_builder() {
        let url = UrlBuilder::new("https://api.example.com")
            .path("users")
            .path("123")
            .query("format", "json")
            .query("limit", "10")
            .build();
        
        assert_eq!(url, "https://api.example.com/users/123?format=json&limit=10");
    }
    
    #[test]
    fn test_url_builder_with_trailing_slash() {
        let url = UrlBuilder::new("https://api.example.com/")
            .path("users")
            .build();
        
        assert_eq!(url, "https://api.example.com/users");
    }
    
    #[test]
    fn test_url_builder_with_spaces() {
        let url = UrlBuilder::new("https://api.example.com")
            .path("search results")
            .query("q", "hello world")
            .build();
        
        assert_eq!(url, "https://api.example.com/search%20results?q=hello%20world");
    }
    
    #[derive(Serialize)]
    struct TestParams {
        name: String,
        age: u32,
        active: bool,
    }
    
    #[test]
    fn test_to_query_params() {
        let params = TestParams {
            name: "John".to_string(),
            age: 30,
            active: true,
        };
        
        let query_params = to_query_params(&params).unwrap();
        assert_eq!(query_params.len(), 3);
        
        // Find each parameter
        assert!(query_params.iter().any(|(k, v)| k == "name" && v == "John"));
        assert!(query_params.iter().any(|(k, v)| k == "age" && v == "30"));
        assert!(query_params.iter().any(|(k, v)| k == "active" && v == "true"));
    }
    
    #[test]
    fn test_url_encode() {
        let encoded = url_encode("hello world & more");
        assert_eq!(encoded, "hello%20world%20%26%20more");
    }
    
    #[test]
    fn test_format_duration() {
        let duration = std::time::Duration::from_millis(1500);
        let formatted = format_duration(duration);
        assert_eq!(formatted, "1.5s");
        
        let duration = std::time::Duration::from_millis(500);
        let formatted = format_duration(duration);
        assert_eq!(formatted, "500ms");
    }
    
    #[test]
    fn test_validate_url() {
        assert!(validate_url("https://api.example.com").is_ok());
        assert!(validate_url("http://localhost:8080/api").is_ok());
        assert!(validate_url("invalid-url").is_err());
        assert!(validate_url("").is_err());
    }
    
    #[test]
    fn test_helper_functions() {
        let _headers = headers();
        let _query = query();
        let _url = url("https://example.com");
        
        // Just test that they compile and can be called
        assert!(true);
    }
}