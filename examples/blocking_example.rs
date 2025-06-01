// examples/blocking_example.rs
#[cfg(feature = "blocking")]
use rusty_http_client::{
    blocking::{BlockingClientConfig, BlockingHttpClient},
    Result,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize)]
struct User {
    name: String,
    email: String,
}

#[derive(Debug, Deserialize)]
struct UserResponse {
    id: u64,
    name: String,
    email: String,
}

#[cfg(feature = "blocking")]
fn run_blocking_example() -> Result<()> {
    println!("=== Blocking HTTP Client Example ===");

    // Create a blocking client with configuration
    let client = BlockingHttpClient::with_config(
        BlockingClientConfig::new()
            .with_base_url("https://jsonplaceholder.typicode.com")
            .with_json_headers()?
            .with_timeout(Duration::from_secs(30))
    )?;

    println!("Created blocking client with base URL: {:?}", client.config().base_url);

    // Example 1: Simple GET request
    println!("\n1. Simple GET request:");
    let response = client.get("/users/1")?;
    println!("Status: {}", response.status());
    
    // Example 2: GET and deserialize JSON
    println!("\n2. GET with JSON deserialization:");
    let user: UserResponse = client.get_json("/users/1")?;
    println!("User: {} ({})", user.name, user.email);

    // Example 3: POST with JSON body
    println!("\n3. POST with JSON body:");
    let new_user = User {
        name: "John Blocking".to_string(),
        email: "john.blocking@example.com".to_string(),
    };
    
    let created_user: UserResponse = client.post_json("/users", &new_user)?;
    println!("Created user: {} with ID {}", created_user.name, created_user.id);

    // Example 4: PUT request
    println!("\n4. PUT request:");
    let updated_user = User {
        name: "John Updated".to_string(),
        email: "john.updated@example.com".to_string(),
    };
    
    let result: UserResponse = client.put_json("/users/1", &updated_user)?;
    println!("Updated user: {}", result.name);

    // Example 5: DELETE request
    println!("\n5. DELETE request:");
    let delete_response = client.delete("/users/1")?;
    println!("Delete status: {}", delete_response.status());

    // Example 6: Custom headers
    println!("\n6. Request with custom headers:");
    let mut headers = std::collections::HashMap::new();
    headers.insert("X-Custom-Header".to_string(), "custom-value".to_string());
    headers.insert("X-Request-ID".to_string(), "12345".to_string());
    
    let custom_response = client.request_with_headers(
        reqwest::Method::GET,
        "/users/2",
        headers,
    )?;
    println!("Custom headers response: {}", custom_response.status());

    // Example 7: Query parameters
    println!("\n7. Request with query parameters:");
    #[derive(Serialize)]
    struct QueryParams {
        _limit: u32,
        _start: u32,
    }
    
    let params = QueryParams {
        _limit: 5,
        _start: 0,
    };
    
    let query_response = client.request_with_query(
        reqwest::Method::GET,
        "/users",
        &params,
    )?;
    println!("Query response status: {}", query_response.status());

    // Example 8: Download bytes
    println!("\n8. Download as bytes:");
    let bytes = client.download_bytes("/users/1")?;
    println!("Downloaded {} bytes", bytes.len());

    // Example 9: Form data
    println!("\n9. Form data submission:");
    #[derive(Serialize)]
    struct FormData {
        title: String,
        body: String,
        #[serde(rename = "userId")]
        user_id: u32,
    }
    
    let form_data = FormData {
        title: "Test Post".to_string(),
        body: "This is a test post".to_string(),
        user_id: 1,
    };
    
    match client.post_form::<_, serde_json::Value>("/posts", &form_data) {
        Ok(response) => println!("Form submission successful: {:?}", response),
        Err(e) => println!("Form submission error: {}", e),
    }

    Ok(())
}

#[cfg(feature = "blocking")]
fn demonstrate_error_handling() -> Result<()> {
    println!("\n=== Error Handling Example ===");
    
    let client = BlockingHttpClient::with_base_url("https://httpbin.org");
    
    // Example 1: 404 Error
    println!("\n1. Handling 404 error:");
    match client.get("/status/404") {
        Ok(response) => println!("Unexpected success: {}", response.status()),
        Err(e) => {
            match e {
                rusty_http_client::HttpError::ResponseError { status, body } => {
                    println!("HTTP Error {}: {}", status, body);
                }
                _ => println!("Other error: {}", e),
            }
        }
    }
    
    // Example 2: Timeout error
    println!("\n2. Handling timeout:");
    let slow_client = BlockingHttpClient::with_config(
        BlockingClientConfig::new()
            .with_base_url("https://httpbin.org")
            .with_timeout(Duration::from_millis(100))
    )?;
    
    match slow_client.get("/delay/2") {
        Ok(_) => println!("Unexpected success"),
        Err(e) => println!("Expected timeout error: {}", e),
    }
    
    // Example 3: JSON parsing error
    println!("\n3. Handling JSON parsing error:");
    match client.get_json::<UserResponse>("/html") {
        Ok(_) => println!("Unexpected success"),
        Err(e) => println!("JSON parsing error: {}", e),
    }
    
    Ok(())
}

#[cfg(feature = "blocking")]
fn demonstrate_file_operations() -> Result<()> {
    println!("\n=== File Operations Example ===");
    
    let client = BlockingHttpClient::with_base_url("https://httpbin.org");
    
    // Download to memory
    println!("\n1. Download to memory:");
    let data = client.download_bytes("/json")?;
    println!("Downloaded {} bytes to memory", data.len());
    
    // Download to file
    println!("\n2. Download to file:");
    let mut file = std::io::Cursor::new(Vec::new());
    let bytes_written = client.download_to_writer("/json", &mut file)?;
    println!("Downloaded {} bytes to writer", bytes_written);
    
    Ok(())
}

#[cfg(not(feature = "blocking"))]
fn main() {
    println!("This example requires the 'blocking' feature to be enabled.");
    println!("Run with: cargo run --example blocking_example --features blocking");
}

#[cfg(feature = "blocking")]
fn main() {
    match run_blocking_example() {
        Ok(_) => {
            if let Err(e) = demonstrate_error_handling() {
                eprintln!("Error handling demo failed: {}", e);
            }
            
            if let Err(e) = demonstrate_file_operations() {
                eprintln!("File operations demo failed: {}", e);
            }
            
            println!("\n=== Blocking Example Completed ===");
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}