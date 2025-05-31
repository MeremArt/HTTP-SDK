# AQUA HTTP Client

A lightweight, ergonomic Rust SDK that wraps around `reqwest` to simplify HTTP requests. Designed as a "no-BS layer" that provides just enough abstraction to make common HTTP tasks easier without hiding the underlying power.

## 🚀 Features

- **Simple HTTP Methods**: `get()`, `post()`, `put()`, `delete()`, `patch()`, `head()`
- **Auto JSON Serialization**: Automatic serialization/deserialization with serde
- **Middleware Support**: Extensible middleware system for auth, logging, and custom processing
- **Blocking & Async**: Both async (default) and blocking clients available
- **Error Handling**: Comprehensive error types with detailed context
- **Utility Builders**: Helper builders for headers, queries, and URLs
- **Connection Pooling**: Built-in connection reuse and pooling
- **Timeout Configuration**: Configurable timeouts and connection settings
- **Base URL Support**: Set base URLs for API clients

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
rusty_http_client = "0.1.0"

# Optional features
rusty_http_client = { version = "0.1.0", features = ["blocking", "middleware"] }
```

## 🏃 Quick Start

### Basic Usage

```rust
use rusty_http_client::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: Option<u64>,
    name: String,
    email: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Create a client with base URL and JSON headers
    let client = HttpClient::with_config(
        ClientConfig::new()
            .with_base_url("https://api.example.com")
            .with_json_headers()?
            .with_timeout(Duration::from_secs(30))
    )?;

    // GET request with JSON deserialization
    let user: User = client.get_json("/users/1").await?;
    println!("User: {} ({})", user.name, user.email);

    // POST with JSON body
    let new_user = User {
        id: None,
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
    };
    let created: User = client.post_json("/users", &new_user).await?;
    println!("Created: {}", created.name);

    Ok(())
}
```

### Using Middleware

```rust
use rusty_http_client::{
    middleware::{AuthMiddleware, LoggingMiddleware},
    prelude::*,
};

let client = HttpClient::new()
    .with_middleware(LoggingMiddleware::new())
    .with_middleware(AuthMiddleware::bearer("your-token"))
    .with_middleware(
        HeaderMiddleware::new()
            .with_header("User-Agent", "MyApp/1.0")
    );

let response = client.get("https://api.example.com/protected").await?;
```

### Blocking Client

```rust
use rusty_http_client::blocking::BlockingHttpClient;

fn main() -> rusty_http_client::Result<()> {
    let client = BlockingHttpClient::with_base_url("https://api.example.com");

    let user: User = client.get_json("/users/1")?;
    println!("User: {}", user.name);

    Ok(())
}
```

## 🔧 Advanced Usage

### Custom Configuration

```rust
let client = HttpClient::with_config(
    ClientConfig::new()
        .with_base_url("https://api.example.com")
        .with_json_headers()?
        .with_timeout(Duration::from_secs(30))
        .with_connect_timeout(Duration::from_secs(10))
        .with_redirects(true, 5)
)?;
```

### Using Utility Builders

```rust
use rusty_http_client::utils::{headers, query, url};

// Build headers
let custom_headers = headers()
    .bearer_auth("token")?
    .header("X-API-Version", "v2")?
    .build();

// Build query parameters
let params = query()
    .param("limit", "10")
    .param("offset", "0")
    .optional_param("filter", Some("active"))
    .build();

// Build URLs
let api_url = url("https://api.example.com")
    .path("v2")
    .path("users")
    .query("limit", "10")
    .build();
```

### Custom Middleware

```rust
use async_trait::async_trait;
use rusty_http_client::middleware::Middleware;

#[derive(Debug)]
struct CustomMiddleware;

#[async_trait]
impl Middleware for CustomMiddleware {
    async fn process_request(&self, request: &mut reqwest::Request) -> rusty_http_client::Result<()> {
        // Add custom logic here
        request.headers_mut().insert("X-Custom", "value".parse().unwrap());
        Ok(())
    }

    async fn process_response(&self, _response: &mut reqwest::Response) -> rusty_http_client::Result<()> {
        // Process response
        Ok(())
    }

    fn name(&self) -> &'static str {
        "CustomMiddleware"
    }
}
```

### Error Handling

```rust
use rusty_http_client::HttpError;

match client.get_json::<User>("/users/1").await {
    Ok(user) => println!("Success: {}", user.name),
    Err(e) => match e {
        HttpError::ResponseError { status, body } => {
            println!("HTTP {}: {}", status, body);
        }
        HttpError::SerializationError(msg) => {
            println!("JSON error: {}", msg);
        }
        HttpError::RequestError(reqwest_err) => {
            println!("Request failed: {}", reqwest_err);
        }
        _ => println!("Other error: {}", e),
    }
}
```

## 📁 Project Structure

```
src/
├── lib.rs           # Main library entry point and re-exports
├── client.rs        # Async HTTP client implementation
├── blocking.rs      # Blocking HTTP client implementation
├── error.rs         # Error types and Result aliases
├── middleware.rs    # Middleware system and built-in middleware
└── utils.rs         # Utility builders and helper functions

examples/
├── basic_usage.rs      # Basic usage examples
├── middleware_example.rs # Middleware usage examples
├── blocking_example.rs   # Blocking client examples
└── advanced_usage.rs    # Advanced patterns and techniques
```

## 🔑 Key Modules

### `client.rs`

- `HttpClient` - Main async HTTP client
- `ClientConfig` - Configuration builder
- HTTP method implementations (GET, POST, PUT, DELETE, etc.)
- Request building and execution with middleware support

### `blocking.rs`

- `BlockingHttpClient` - Synchronous HTTP client
- `BlockingClientConfig` - Configuration for blocking client
- Same API as async client but without async/await

### `middleware.rs`

- `Middleware` trait for custom middleware
- `AuthMiddleware` - Authentication (Bearer, Basic, API Key)
- `LoggingMiddleware` - Request/response logging
- `HeaderMiddleware` - Custom header injection
- `RetryMiddleware` - Request retry logic

### `error.rs`

- `HttpError` - Comprehensive error types
- `Result<T>` - Type alias for `std::result::Result<T, HttpError>`
- Error conversion from reqwest and serde errors

### `utils.rs`

- `HeaderBuilder` - Fluent header construction
- `QueryBuilder` - Query parameter building
- `UrlBuilder` - URL construction with path/query support
- Helper functions for common operations

## 🎯 Use Cases

Perfect for:

- **API Clients**: Building SDKs for REST APIs
- **Microservices**: Service-to-service communication
- **CLI Tools**: Command-line applications making HTTP requests
- **Testing**: HTTP testing and mocking
- **Learning**: Understanding Rust HTTP client patterns

## 🔄 Migration from Raw Reqwest

Before (raw reqwest):

```rust
let client = reqwest::Client::new();
let response = client
    .post("https://api.example.com/users")
    .header("Content-Type", "application/json")
    .header("Authorization", "Bearer token")
    .json(&user)
    .send()
    .await?;

if response.status().is_success() {
    let created_user: User = response.json().await?;
    println!("Created: {}", created_user.name);
} else {
    eprintln!("Error: {}", response.status());
}
```

After (rusty_http_client):

```rust
let client = HttpClient::with_base_url("https://api.example.com")
    .with_middleware(AuthMiddleware::bearer("token"));

let created_user: User = client.post_json("/users", &user).await?;
println!("Created: {}", created_user.name);
```

## 📊 Performance

- **Connection Pooling**: Automatic connection reuse
- **Async by Default**: Built on tokio for high concurrency
- **Minimal Overhead**: Thin wrapper around reqwest
- **Configurable Timeouts**: Fine-tune performance characteristics

## 🧪 Testing

Run the test suite:

```bash
# Run all tests
cargo test

# Run with all features
cargo test --all-features

# Run specific test module
cargo test --lib client

# Run examples
cargo run --example basic_usage
cargo run --example middleware_example
cargo run --example blocking_example --features blocking
cargo run --example advanced_usage
```

## 📝 Examples

Check out the `examples/` directory for comprehensive usage examples:

- **basic_usage.rs** - Getting started examples
- **middleware_example.rs** - Middleware system usage
- **blocking_example.rs** - Synchronous client usage
- **advanced_usage.rs** - Advanced patterns and performance tips

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built on top of the excellent [reqwest](https://github.com/seanmonstar/reqwest) library
- Inspired by the need for ergonomic HTTP clients in Rust
- Thanks to the Rust community for feedback and contributions
