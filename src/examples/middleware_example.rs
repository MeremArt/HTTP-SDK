
use rusty_http_client::{
    middleware::{AuthMiddleware, HeaderMiddleware, LoggingMiddleware},
    ClientConfig, HttpClient, Result,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

#[derive(Debug, Deserialize)]
struct User {
    id: u64,
    name: String,
    email: String,
    created_at: String,
}

async fn run_middleware_example() -> Result<()> {
    // Initialize logging
    env_logger::init();

    // Create a client with multiple middleware
    let client = HttpClient::with_config(
        ClientConfig::new()
            .with_base_url("https://jsonplaceholder.typicode.com")
            .with_json_headers()?
            .with_timeout(Duration::from_secs(30))
    )?
    .with_middleware(LoggingMiddleware::new())
    .with_middleware(AuthMiddleware::bearer("your-api-token"))
    .with_middleware(
        HeaderMiddleware::new()
            .with_header("X-Client-Version", "1.0.0")
            .with_header("X-Request-Source", "middleware-example")
    );

    println!("=== Middleware Example ===");
    println!("Client has {} middleware(s) configured", client.middleware_count());

    // Example 1: GET request with middleware processing
    println!("\n1. GET request with middleware:");
    match client.get_json::<Vec<User>>("/users?_limit=3").await {
        Ok(users) => {
            println!("Retrieved {} users:", users.len());
            for user in users.iter().take(2) {
                println!("  - {} ({})", user.name, user.email);
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    // Example 2: POST request with middleware
    println!("\n2. POST request with middleware:");
    let new_user = CreateUserRequest {
        name: "John Middleware".to_string(),
        email: "john.middleware@example.com".to_string(),
    };

    match client.post_json::<_, User>("/users", &new_user).await {
        Ok(user) => {
            println!("Created user: {} with ID {}", user.name, user.id);
        }
        Err(e) => println!("Error creating user: {}", e),
    }

    // Example 3: Using different authentication middleware
    println!("\n3. Client with API key middleware:");
    let api_key_client = HttpClient::with_config(
        ClientConfig::new()
            .with_base_url("https://api.example.com")
            .with_json_headers()?
    )?
    .with_middleware(AuthMiddleware::api_key("X-API-Key", "your-secret-api-key"))
    .with_middleware(LoggingMiddleware::requests_only());

    println!("API key client configured with {} middleware(s)", api_key_client.middleware_count());

    // Example 4: Custom headers middleware
    println!("\n4. Custom headers example:");
    let custom_client = HttpClient::new()
        .with_middleware(
            HeaderMiddleware::new()
                .with_header("User-Agent", "RustyHttpClient/1.0")
                .with_header("Accept-Language", "en-US,en;q=0.9")
                .with_header("X-Custom-Header", "custom-value")
        );

    // This would make a request with all the custom headers
    println!("Custom client ready with custom headers middleware");

    Ok(())
}

async fn demonstrate_auth_types() -> Result<()> {
    println!("\n=== Authentication Middleware Types ===");

    // Bearer token authentication
    let bearer_client = HttpClient::new()
        .with_middleware(AuthMiddleware::bearer("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."));
    println!("1. Bearer token auth configured");

    // Basic authentication
    let basic_client = HttpClient::new()
        .with_middleware(AuthMiddleware::basic("dXNlcjpwYXNzd29yZA=="));
    println!("2. Basic auth configured");

    // API key authentication
    let api_key_client = HttpClient::new()
        .with_middleware(AuthMiddleware::api_key("X-RapidAPI-Key", "your-rapidapi-key"));
    println!("3. API key auth configured");

    Ok(())
}

async fn demonstrate_logging_options() -> Result<()> {
    println!("\n=== Logging Middleware Options ===");

    // Log everything
    let full_logging_client = HttpClient::new()
        .with_middleware(LoggingMiddleware::new());
    println!("1. Full logging (requests + responses)");

    // Log only requests
    let request_logging_client = HttpClient::new()
        .with_middleware(LoggingMiddleware::requests_only());
    println!("2. Request-only logging");

    // Log only responses
    let response_logging_client = HttpClient::new()
        .with_middleware(LoggingMiddleware::responses_only());
    println!("3. Response-only logging");

    Ok(())
}

#[tokio::main]
async fn main() {
    match run_middleware_example().await {
        Ok(_) => {
            if let Err(e) = demonstrate_auth_types().await {
                eprintln!("Auth demo error: {}", e);
            }
            
            if let Err(e) = demonstrate_logging_options().await {
                eprintln!("Logging demo error: {}", e);
            }
            
            println!("\n=== Middleware Example Completed ===");
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}