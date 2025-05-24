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

