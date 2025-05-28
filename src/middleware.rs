use crate::error::{HttpError, Result};
use reqwest::{Request, Response};
use std::fmt;

#[async_trait::async_trait]
pub trait Middleware: Send + Sync + fmt::Debug {
    /// Process the request before it's sent
    async fn process_request(&self, request: &mut Request) -> Result<()>;
    
    /// Process the response after it's received
    async fn process_response(&self, response: &mut Response) -> Result<()>;
    
    /// Get the name of this middleware for debugging
    fn name(&self) -> &'static str;
}

#[derive(Debug, Clone)]
pub enum AuthType {
Bearer,
Basic,
ApiKey(String), // holds header name, different APIs may use things like,x-access-token,X-API-Key 
}

#[derive(Debug, Clone)]
pub struct AuthMiddleware {
    pub token: String,
    pub auth_type: AuthType,
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
            //HTTP headers are ultimately byte-based on the wire (protocol level).
            AuthType::ApiKey(header_name) => {
                let header_name = reqwest::header::HeaderName::from_bytes(header_name.as_bytes())
                    .map_err(|_| {
                        HttpError::MiddlewareError(format!("Invalid header name: {}", header_name))
                    })?;
                //HTTP is ultimately a byte protocol
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

    // async fn process_response(&self, response: &mut Response) -> Result<()> {
    //     println!(
    //         "[{}] Response status: {}",
    //         self.name(),
    //         response.status()
    //     );
    
    //     Ok(())
    // }
    
    
    fn name(&self) -> &'static str {
        "AuthMiddleware"
    }
}
