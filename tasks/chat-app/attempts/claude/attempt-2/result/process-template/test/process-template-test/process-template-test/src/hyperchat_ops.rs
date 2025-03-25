use crate::*;
use hyperware_process_lib::{Address, Request, Response, LazyLoadBlob};
use serde_json::{json, to_vec, to_string, Value};
use shared_types::{ApiRequest, WebSocketMessage};

/// Run a series of tests for the Hyperchat application
pub fn run_hyperchat_tests(log_file: &mut File, client_addresses: &Vec<Address>) -> anyhow::Result<()> {
    write_log(log_file, "Starting Hyperchat tests")?;
    
    for client in client_addresses.iter() {
        write_log(log_file, &format!("Testing client: {}", client))?;
        
        // Test HTTP API endpoints
        test_http_api(client, log_file)?;
        
        // Test sending external messages
        test_external_messages(client, log_file)?;
        
        write_log(log_file, &format!("Completed tests for client: {}", client))?;
    }
    
    write_log(log_file, "All Hyperchat tests completed successfully")?;
    Ok(())
}

/// Test HTTP API endpoints
fn test_http_api(client: &Address, log_file: &mut File) -> anyhow::Result<()> {
    write_log(log_file, "Testing HTTP API endpoints")?;
    
    // Test /api endpoint
    let api_response = send_request(client, "/api", "GET", None)?;
    write_log(log_file, &format!("API response: {}", api_response))?;
    
    // Verify the response
    let response_json: Value = serde_json::from_str(&api_response)?;
    if !response_json["status"].is_string() || response_json["status"].as_str().unwrap() != "ok" {
        write_log(log_file, &format!("Error: Expected 'status' field with value 'ok', got {:?}", response_json))?;
        return Err(anyhow::anyhow!("API response validation failed"));
    }
    
    write_log(log_file, "HTTP API tests passed")?;
    Ok(())
}

/// Test sending external messages
fn test_external_messages(client: &Address, log_file: &mut File) -> anyhow::Result<()> {
    write_log(log_file, "Testing external messages")?;
    
    // Test sending a plain text message
    let text_message = "Hello from test client!";
    let text_response = send_external_message(client, text_message)?;
    write_log(log_file, &format!("Text message response: {}", text_response))?;
    
    // Verify response
    let text_response_json: Value = serde_json::from_str(&text_response)?;
    if !text_response_json["status"].is_string() || text_response_json["status"].as_str().unwrap() != "ok" {
        write_log(log_file, &format!("Error: Expected 'status' field with value 'ok', got {:?}", text_response_json))?;
        return Err(anyhow::anyhow!("Text message response validation failed"));
    }
    
    // Test sending a JSON message
    let json_message = json!({
        "type": "chat_message",
        "conversation_id": "test-conv-123",
        "content": "Test message from external system",
        "sender": "test-client"
    });
    
    let json_response = send_external_message(client, &json_message.to_string())?;
    write_log(log_file, &format!("JSON message response: {}", json_response))?;
    
    // Verify response
    let json_response_json: Value = serde_json::from_str(&json_response)?;
    if !json_response_json["status"].is_string() || json_response_json["status"].as_str().unwrap() != "ok" {
        write_log(log_file, &format!("Error: Expected 'status' field with value 'ok', got {:?}", json_response_json))?;
        return Err(anyhow::anyhow!("JSON message response validation failed"));
    }
    
    write_log(log_file, "External message tests passed")?;
    Ok(())
}

/// Send an HTTP request to the specified path
fn send_request(client: &Address, path: &str, method: &str, body: Option<String>) -> anyhow::Result<String> {
    // Create HTTP request body
    let request_body = json!({
        "path": path,
        "method": method,
        "headers": {
            "Content-Type": "application/json"
        },
        "body": body.unwrap_or_default()
    });
    
    // Send request
    let response = Request::to(client.clone())
        .body(to_vec(&request_body)?)
        .send_and_await_response(10)??
        .body()
        .to_vec();
    
    // Parse response
    match String::from_utf8(response) {
        Ok(response_str) => Ok(response_str),
        Err(_) => Err(anyhow::anyhow!("Failed to parse response as string"))
    }
}

/// Send an external message to the client
fn send_external_message(client: &Address, message: &str) -> anyhow::Result<String> {
    // Send message
    let response = Request::to(client.clone())
        .body(message.as_bytes().to_vec())
        .send_and_await_response(10)??
        .body()
        .to_vec();
    
    // Parse response
    match String::from_utf8(response) {
        Ok(response_str) => Ok(response_str),
        Err(_) => Err(anyhow::anyhow!("Failed to parse response as string"))
    }
}