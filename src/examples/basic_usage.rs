
use rusty_http_client::{HttpClient, Result, ClientConfig};
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

async fn run_example() -> Result<()> {
    // Create a client with base URL and default headers
    let client = HttpClient::with_config(
        ClientConfig::new()
            .with_base_url("https://jsonplaceholder.typicode.com")
            .with_json_headers()?
            .with_timeout(Duration::from_secs(30))
    )?;

    println!("=== Basic HTTP Client Example ===");

    // Simple GET request
    println!("\n1. Simple GET request:");
    let response = client.get("/users").await?;
    println!("Status: {}", response.status());

    // GET with JSON deserialization
    println!("\n2. GET with JSON deserialization:");
    let user: UserResponse = client.get_json("/users/1").await?;
    println!("User: {} ({})", user.name, user.email);

    // POST with JSON body
    println!("\n3. POST with JSON body:");
    let new_user = User {
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
    };
    let created_user: UserResponse = client.post_json("/users", &new_user).await?;
    println!("Created user: {} with ID {}", created_user.name, created_user.id);

    Ok(())
}

#[tokio::main]
async fn main() {
    match run_example().await {
        Ok(_) => println!("\n=== Example completed successfully! ==="),
        Err(e) => eprintln!("Error: {}", e),
    }
}