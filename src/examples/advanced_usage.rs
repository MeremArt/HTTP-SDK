// examples/advanced_usage.rs
use rusty_http_client::{
    middleware::{AuthMiddleware, HeaderMiddleware, LoggingMiddleware},
    utils::{headers, query, url},
    ClientConfig, HttpClient, Result,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Duration};

#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse<T> {
    data: T,
    message: String,
    success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: Option<u64>,
    name: String,
    email: String,
    age: Option<u32>,
}

#[derive(Debug, Serialize)]
struct SearchParams {
    q: String,
    limit: u32,
    offset: u32,
    sort: String,
}

async fn run_advanced_example() -> Result<()> {
    println!("=== Advanced Usage Example ===");

    // Create a sophisticated client with multiple configurations
    let client = HttpClient::with_config(
        ClientConfig::new()
            .with_base_url("https://jsonplaceholder.typicode.com")
            .with_json_headers()?
            .with_timeout(Duration::from_secs(30))
            .with_connect_timeout(Duration::from_secs(10))
            .with_redirects(true, 5)
    )?
    .with_middleware(LoggingMiddleware::new())
    .with_middleware(AuthMiddleware::bearer("advanced-token"))
    .with_middleware(
        HeaderMiddleware::new()
            .with_header("X-Client-Name", "AdvancedExample")
            .with_header("X-Client-Version", "2.0.0")
            .with_header("Accept-Encoding", "gzip, deflate")
    );

    // Example 1: Using utility builders
    println!("\n1. Using utility builders:");
    
    let custom_headers = headers()
        .header("X-Request-ID", "req-12345")?
        .header("X-Correlation-ID", "corr-67890")?
        .user_agent("AdvancedClient/2.0")?
        .build();
    
    let search_url = url("https://api.example.com")
        .path("v2")
        .path("search")
        .query("type", "users")
        .query("active", "true")
        .build();
    
    println!("Built URL: {}", search_url);
    println!("Custom headers count: {}", custom_headers.len());

    // Example 2: Complex query parameters
    println!("\n2. Complex query handling:");
    
    let search_params = SearchParams {
        q: "rust developer".to_string(),
        limit: 20,
        offset: 0,
        sort: "created_desc".to_string(),
    };
    
    let query_params = query()
        .param("category", "programming")
        .param("verified", "true")
        .optional_param("location", Some("remote"))
        .optional_param("salary_min", None::<String>)
        .build();
    
    println!("Query params: {:?}", query_params);

    // Example 3: Advanced request building with the client
    println!("\n3. Advanced request patterns:");
    
    // Using request builder pattern
    let users_response = client
        .request_with_query(reqwest::Method::GET, "/users", &search_params)
        .await?;
    
    println!("Search response status: {}", users_response.status());

    // Example 4: Conditional request building
    println!("\n4. Conditional request building:");
    
    let include_details = true;
    let user_id = 1;
    
    let mut request_url = format!("/users/{}", user_id);
    let mut query_builder = query();
    
    if include_details {
        query_builder = query_builder.param("include", "details,posts");
    }
    
    let query_string = query_builder.build_query_string();
    if !query_string.is_empty() {
        request_url.push_str(&query_string);
    }
    
    let detailed_user: User = client.get_json(&request_url).await?;
    println!("Detailed user: {} ({})", detailed_user.name, detailed_user.email);

    // Example 5: Bulk operations
    println!("\n5. Bulk operations:");
    
    let user_ids = vec![1, 2, 3, 4, 5];
    let mut users = Vec::new();
    
    for id in user_ids {
        match client.get_json::<User>(&format!("/users/{}", id)).await {
            Ok(user) => users.push(user),
            Err(e) => println!("Failed to fetch user {}: {}", id, e),
        }
    }
    
    println!("Successfully fetched {} users", users.len());

    // Example 6: Request retry with custom logic
    println!("\n6. Custom retry logic:");
    
    let max_retries = 3;
    let mut attempt = 0;
    
    loop {
        attempt += 1;
        
        match client.get("/users/999").await {
            Ok(response) => {
                println!("Success on attempt {}: {}", attempt, response.status());
                break;
            }
            Err(e) => {
                if attempt >= max_retries {
                    println!("Failed after {} attempts: {}", max_retries, e);
                    break;
                } else {
                    println!("Attempt {} failed, retrying: {}", attempt, e);
                    tokio::time::sleep(Duration::from_millis(100 * attempt as u64)).await;
                }
            }
        }
    }

    Ok(())
}

async fn demonstrate_advanced_patterns() -> Result<()> {
    println!("\n=== Advanced Patterns ===");

    let client = HttpClient::new();

    // Pattern 1: Resource-specific clients
    println!("\n1. Resource-specific clients:");
    
    let users_client = HttpClient::with_config(
        ClientConfig::new()
            .with_base_url("https://jsonplaceholder.typicode.com/users")
            .with_json_headers()?
    )?;
    
    let posts_client = HttpClient::with_config(
        ClientConfig::new()
            .with_base_url("https://jsonplaceholder.typicode.com/posts")
            .with_json_headers()?
    )?;
    
    // Now we can use shorter paths
    let user: User = users_client.get_json("/1").await?;
    println!("User from users client: {}", user.name);
    
    // Pattern 2: Request templates
    println!("\n2. Request templates:");
    
    struct ApiClient {
        client: HttpClient,
    }
    
    impl ApiClient {
        fn new(base_url: &str, api_key: &str) -> Result<Self> {
            let client = HttpClient::with_config(
                ClientConfig::new()
                    .with_base_url(base_url)
                    .with_json_headers()?
            )?
            .with_middleware(AuthMiddleware::api_key("X-API-Key", api_key));
            
            Ok(Self { client })
        }
        
        async fn get_user(&self, id: u64) -> Result<User> {
            self.client.get_json(&format!("/users/{}", id)).await
        }
        
        async fn create_user(&self, user: &User) -> Result<User> {
            self.client.post_json("/users", user).await
        }
        
        async fn search_users(&self, query: &str, limit: u32) -> Result<Vec<User>> {
            let params = query()
                .param("q", query)
                .param("limit", limit.to_string())
                .build();
            
            let response = self.client
                .request_with_query(reqwest::Method::GET, "/users/search", &params)
                .await?;
            
            // Process the response manually since it might have a different structure
            if response.status().is_success() {
                let text = response.text().await.map_err(rusty_http_client::HttpError::from)?;
                serde_json::from_str(&text).map_err(|e| {
                    rusty_http_client::HttpError::SerializationError(e.to_string())
                })
            } else {
                Err(rusty_http_client::HttpError::ResponseError {
                    status: response.status(),
                    body: "Search failed".to_string(),
                })
            }
        }
    }
    
    let api_client = ApiClient::new("https://jsonplaceholder.typicode.com", "test-key")?;
    let template_user = api_client.get_user(1).await?;
    println!("Template user: {}", template_user.name);

    // Pattern 3: Response processing pipeline
    println!("\n3. Response processing pipeline:");
    
    async fn process_response<T, F, R>(
        client: &HttpClient,
        url: &str,
        processor: F,
    ) -> Result<R>
    where
        F: FnOnce(reqwest::Response) -> Result<R>,
    {
        let response = client.get(url).await?;
        processor(response)
    }
    
    let processed_result = process_response::<(), _, String>(&client, "https://httpbin.org/json", |response| {
        // Custom processing logic
        let status = response.status();
        if status.is_success() {
            Ok(format!("Success: {}", status))
        } else {
            Err(rusty_http_client::HttpError::ResponseError {
                status,
                body: "Processing failed".to_string(),
            })
        }
    }).await?;
    
    println!("Processed result: {}", processed_result);

    Ok(())
}

async fn demonstrate_performance_patterns() -> Result<()> {
    println!("\n=== Performance Patterns ===");

    // Pattern 1: Connection pooling and reuse
    println!("\n1. Connection pooling:");
    
    let high_performance_client = HttpClient::with_config(
        ClientConfig::new()
            .with_base_url("https://jsonplaceholder.typicode.com")
            .with_json_headers()?
            .with_timeout(Duration::from_secs(30))
            .with_connect_timeout(Duration::from_secs(5))
    )?;
    
    // Make multiple requests to demonstrate connection reuse
    let start = std::time::Instant::now();
    
    let mut tasks = Vec::new();
    for i in 1..=5 {
        let client = high_performance_client.clone();
        let task = tokio::spawn(async move {
            client.get_json::<User>(&format!("/users/{}", i)).await
        });
        tasks.push(task);
    }
    
    let results = futures::future::join_all(tasks).await;
    let successful_requests = results.iter().filter(|r| r.is_ok()).count();
    
    let duration = start.elapsed();
    println!("Completed {} concurrent requests in {:?}", successful_requests, duration);

    // Pattern 2: Request batching
    println!("\n2. Request batching:");
    
    async fn batch_get_users(client: &HttpClient, ids: Vec<u64>) -> Vec<Result<User>> {
        let tasks: Vec<_> = ids.into_iter().map(|id| {
            let client = client.clone();
            tokio::spawn(async move {
                client.get_json::<User>(&format!("/users/{}", id)).await
            })
        }).collect();
        
        let results = futures::future::join_all(tasks).await;
        results.into_iter().map(|r| r.unwrap()).collect()
    }
    
    let user_ids = vec![1, 2, 3, 4, 5];
    let batch_results = batch_get_users(&high_performance_client, user_ids).await;
    let successful_batch = batch_results.iter().filter(|r| r.is_ok()).count();
    println!("Batch operation: {}/{} successful", successful_batch, batch_results.len());

    // Pattern 3: Streaming responses (for large data)
    println!("\n3. Streaming pattern simulation:");
    
    let streaming_client = HttpClient::new();
    let response = streaming_client.get("https://httpbin.org/stream/10").await?;
    
    if response.status().is_success() {
        let text = response.text().await.map_err(rusty_http_client::HttpError::from)?;
        let lines: Vec<&str> = text.lines().take(3).collect(); // Take first 3 lines
        println!("Streamed {} lines (showing first 3)", lines.len());
        for (i, line) in lines.iter().enumerate() {
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(line) {
                println!("  Line {}: {}", i + 1, json_value.get("id").unwrap_or(&serde_json::Value::Null));
            }
        }
    }

    Ok(())
}

async fn demonstrate_error_recovery() -> Result<()> {
    println!("\n=== Error Recovery Patterns ===");

    let client = HttpClient::with_base_url("https://httpbin.org");

    // Pattern 1: Graceful degradation
    println!("\n1. Graceful degradation:");
    
    async fn get_user_with_fallback(client: &HttpClient, id: u64) -> User {
        match client.get_json::<User>(&format!("/users/{}", id)).await {
            Ok(user) => user,
            Err(_) => {
                // Fallback to default user
                User {
                    id: Some(id),
                    name: "Unknown User".to_string(),
                    email: "unknown@example.com".to_string(),
                    age: None,
                }
            }
        }
    }
    
    let fallback_user = get_user_with_fallback(&client, 999).await;
    println!("Fallback user: {}", fallback_user.name);

    // Pattern 2: Circuit breaker simulation
    println!("\n2. Circuit breaker pattern:");
    
    struct SimpleCircuitBreaker {
        failure_count: std::sync::Arc<std::sync::Mutex<u32>>,
        threshold: u32,
    }
    
    impl SimpleCircuitBreaker {
        fn new(threshold: u32) -> Self {
            Self {
                failure_count: std::sync::Arc::new(std::sync::Mutex::new(0)),
                threshold,
            }
        }
        
        async fn execute<F, T>(&self, operation: F) -> Result<T>
        where
            F: std::future::Future<Output = Result<T>>,
        {
            let current_failures = *self.failure_count.lock().unwrap();
            
            if current_failures >= self.threshold {
                return Err(rusty_http_client::HttpError::ConfigError(
                    "Circuit breaker is open".to_string()
                ));
            }
            
            match operation.await {
                Ok(result) => {
                    // Reset failure count on success
                    *self.failure_count.lock().unwrap() = 0;
                    Ok(result)
                }
                Err(e) => {
                    // Increment failure count
                    *self.failure_count.lock().unwrap() += 1;
                    Err(e)
                }
            }
        }
    }
    
    let circuit_breaker = SimpleCircuitBreaker::new(3);
    
    for i in 1..=5 {
        let result = circuit_breaker.execute(async {
            client.get("/status/500").await
        }).await;
        
        match result {
            Ok(_) => println!("Request {} succeeded", i),
            Err(e) => println!("Request {} failed: {}", i, e),
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    // Enable logging
    env_logger::init();

    match run_advanced_example().await {
        Ok(_) => {
            if let Err(e) = demonstrate_advanced_patterns().await {
                eprintln!("Advanced patterns demo failed: {}", e);
            }
            
            if let Err(e) = demonstrate_performance_patterns().await {
                eprintln!("Performance patterns demo failed: {}", e);
            }
            
            if let Err(e) = demonstrate_error_recovery().await {
                eprintln!("Error recovery demo failed: {}", e);
            }
            
            println!("\n=== Advanced Usage Example Completed ===");
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}