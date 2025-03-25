use anyhow::{anyhow, Result};
use hyperware_process_lib::{
    http::server::{HttpRequest, HttpServer},
    logging::{debug, error, info},
    Address, Message,
};
use serde_json::json;
use shared_types::{
    ApiRequest, ApiResponse, AppState, ChatMessage,
    MessageChannel, MessageType, WebSocketMessage,
};

use crate::database::{ContactRepository, ConversationRepository, MessageRepository, get_timestamp};
use crate::message_handlers::{broadcast_to_websocket_clients, get_user_channels};

pub fn handle_http_request(
    our: &Address,
    body: &[u8],
    state: &mut AppState,
    server: &mut HttpServer,
) -> Result<()> {
    let http_req = serde_json::from_slice::<HttpRequest>(body)?;
    
    // Log HTTP request
    info!("HTTP Request: {} {}", http_req.method, http_req.path);
    info!("HTTP Body: {:?}", String::from_utf8_lossy(&http_req.body));
    
    let response = match (http_req.method.as_str(), http_req.path.as_str()) {
        // Status endpoint
        ("GET", "/api/status") => handle_status(state),
        
        // Contacts endpoints
        ("GET", "/api/contacts") => handle_get_contacts(),
        ("POST", "/api/contacts") => {
            info!("Adding contact with body: {:?}", String::from_utf8_lossy(&http_req.body));
            handle_add_contact(&http_req)
        },
        ("DELETE", path) if path.starts_with("/api/contacts/") => {
            let id = extract_id(path, "/api/contacts/")?;
            handle_delete_contact(id)
        },
        
        // Conversations endpoints
        ("GET", "/api/conversations") => {
            // Extract user from query params
            let user_address = http_req.params.get("user").ok_or_else(|| anyhow!("User address is required"))?;
            handle_get_conversations(user_address)
        },
        ("POST", "/api/conversations") => handle_create_conversation(&http_req),
        ("GET", path) if path.starts_with("/api/conversations/") && path.ends_with("/messages") => {
            let conversation_id = extract_id_from_middle(path, "/api/conversations/", "/messages")?;
            handle_get_messages(conversation_id)
        },
        ("POST", path) if path.starts_with("/api/conversations/") && path.ends_with("/messages") => {
            let conversation_id = extract_id_from_middle(path, "/api/conversations/", "/messages")?;
            handle_send_message(conversation_id, &http_req, state, server)
        },
        ("POST", path) if path.starts_with("/api/conversations/") && path.ends_with("/members") => {
            let conversation_id = extract_id_from_middle(path, "/api/conversations/", "/members")?;
            handle_add_member(conversation_id, &http_req)
        },
        
        // Message operations
        ("PUT", path) if path.starts_with("/api/messages/") && path.ends_with("/read") => {
            let message_id = extract_id_from_middle(path, "/api/messages/", "/read")?;
            handle_mark_message_read(message_id)
        },
        
        // Return 404 for unknown paths
        _ => ApiResponse::Error {
            code: 404,
            message: format!("Not found: {}", http_req.path),
        },
    };
    
    // Send response
    let response_json = serde_json::to_string(&response)?;
    server.http_respond(&http_req.channel_id, 200, "application/json", response_json.as_bytes())?;
    
    Ok(())
}

// Status endpoint handler
fn handle_status(state: &AppState) -> ApiResponse {
    let mut counts_by_channel = std::collections::HashMap::new();
    
    for (channel, count) in &state.message_counts {
        counts_by_channel.insert(format!("{:?}", channel), *count);
    }
    
    ApiResponse::Status {
        connected_clients: state.connected_clients.len(),
        message_count: state.message_history.len(),
        message_counts_by_channel: counts_by_channel,
    }
}

// Contact endpoints handlers
fn handle_get_contacts() -> ApiResponse {
    match ContactRepository::get_contacts() {
        Ok(contacts) => ApiResponse::Contacts { contacts },
        Err(e) => ApiResponse::Error {
            code: 500,
            message: format!("Failed to get contacts: {}", e),
        },
    }
}

fn handle_add_contact(req: &HttpRequest) -> ApiResponse {
    #[derive(serde::Deserialize, Debug)]
    struct AddContactRequest {
        name: String,
        node_address: String,
    }
    
    info!("Processing add contact request with body: {:?}", String::from_utf8_lossy(&req.body));
    
    // Try to parse the JSON body
    let add_request = match serde_json::from_slice::<AddContactRequest>(&req.body) {
        Ok(req) => {
            info!("Successfully parsed request: {:?}", req);
            req
        },
        Err(e) => {
            error!("Failed to parse request: {}", e);
            return ApiResponse::Error {
                code: 400,
                message: format!("Invalid request body: {}", e),
            };
        },
    };
    
    // Validate the request
    if add_request.name.trim().is_empty() {
        return ApiResponse::Error {
            code: 400,
            message: "Contact name cannot be empty".to_string(),
        };
    }
    
    if add_request.node_address.trim().is_empty() {
        return ApiResponse::Error {
            code: 400,
            message: "Node address cannot be empty".to_string(),
        };
    }
    
    // Try to add the contact
    match ContactRepository::add_contact(&add_request.name, &add_request.node_address) {
        Ok(id) => {
            info!("Contact added successfully with ID: {}", id);
            ApiResponse::Success {
                message: format!("Contact added with ID: {}", id),
            }
        },
        Err(e) => {
            error!("Failed to add contact: {}", e);
            ApiResponse::Error {
                code: 500,
                message: format!("Failed to add contact: {}", e),
            }
        },
    }
}

fn handle_delete_contact(id: i64) -> ApiResponse {
    match ContactRepository::delete_contact(id) {
        Ok(_) => ApiResponse::Success {
            message: "Contact deleted".to_string(),
        },
        Err(e) => ApiResponse::Error {
            code: 500,
            message: format!("Failed to delete contact: {}", e),
        },
    }
}

// Conversation endpoints handlers
fn handle_get_conversations(user_address: &str) -> ApiResponse {
    match ConversationRepository::get_conversations_for_user(user_address) {
        Ok(conversations) => ApiResponse::Conversations { conversations },
        Err(e) => ApiResponse::Error {
            code: 500,
            message: format!("Failed to get conversations: {}", e),
        },
    }
}

fn handle_create_conversation(req: &HttpRequest) -> ApiResponse {
    #[derive(serde::Deserialize)]
    struct CreateConversationRequest {
        title: Option<String>,
        members: Vec<String>,
        is_group: bool,
    }
    
    let create_request = match serde_json::from_slice::<CreateConversationRequest>(&req.body) {
        Ok(req) => req,
        Err(_) => return ApiResponse::Error {
            code: 400,
            message: "Invalid request body".to_string(),
        },
    };
    
    let conversation_type = if create_request.is_group { "group" } else { "direct" };
    
    match ConversationRepository::create_conversation(
        conversation_type,
        create_request.title.as_deref(),
        &create_request.members,
    ) {
        Ok(id) => ApiResponse::Success {
            message: format!("Conversation created with ID: {}", id),
        },
        Err(e) => ApiResponse::Error {
            code: 500,
            message: format!("Failed to create conversation: {}", e),
        },
    }
}

fn handle_get_messages(conversation_id: &str) -> ApiResponse {
    match MessageRepository::get_messages(conversation_id) {
        Ok(messages) => ApiResponse::Messages { messages },
        Err(e) => ApiResponse::Error {
            code: 500,
            message: format!("Failed to get messages: {}", e),
        },
    }
}

fn handle_send_message(
    conversation_id: &str,
    req: &HttpRequest,
    state: &mut AppState,
    server: &mut HttpServer,
) -> ApiResponse {
    #[derive(serde::Deserialize)]
    struct SendMessageRequest {
        sender_id: String,
        content: String,
        has_attachment: bool,
        attachment_path: Option<String>,
    }
    
    let send_request = match serde_json::from_slice::<SendMessageRequest>(&req.body) {
        Ok(req) => req,
        Err(_) => return ApiResponse::Error {
            code: 400,
            message: "Invalid request body".to_string(),
        },
    };
    
    // Add message to database
    let message_id = match MessageRepository::add_message(
        conversation_id,
        &send_request.sender_id,
        &send_request.content,
        send_request.has_attachment,
        send_request.attachment_path.as_deref(),
    ) {
        Ok(id) => id,
        Err(e) => return ApiResponse::Error {
            code: 500,
            message: format!("Failed to send message: {}", e),
        },
    };
    
    // Get conversation members to notify them
    let members = match ConversationRepository::get_conversation_members(conversation_id) {
        Ok(members) => members,
        Err(e) => return ApiResponse::Error {
            code: 500,
            message: format!("Failed to get conversation members: {}", e),
        },
    };
    
    // Create WebSocket message for real-time notification
    let ws_message = WebSocketMessage::ChatMessage {
        conversation_id: conversation_id.to_string(),
        sender_id: send_request.sender_id.clone(),
        content: send_request.content.clone(),
        timestamp: get_timestamp(),
        has_attachment: send_request.has_attachment,
        attachment_path: send_request.attachment_path.clone(),
    };
    
    let ws_message_json = match serde_json::to_string(&ws_message) {
        Ok(json) => json,
        Err(e) => return ApiResponse::Error {
            code: 500,
            message: format!("Failed to serialize WebSocket message: {}", e),
        },
    };
    
    // Notify all members who are connected via WebSocket
    for member in members {
        let user_channels = get_user_channels(state, &member.member_address);
        for channel_id in user_channels {
            if let Err(e) = server.ws_push(channel_id, &ws_message_json) {
                error!("Failed to push WebSocket message: {}", e);
            }
        }
    }
    
    ApiResponse::Success {
        message: format!("Message sent with ID: {}", message_id),
    }
}

fn handle_add_member(conversation_id: &str, req: &HttpRequest) -> ApiResponse {
    #[derive(serde::Deserialize)]
    struct AddMemberRequest {
        node_address: String,
    }
    
    let add_request = match serde_json::from_slice::<AddMemberRequest>(&req.body) {
        Ok(req) => req,
        Err(_) => return ApiResponse::Error {
            code: 400,
            message: "Invalid request body".to_string(),
        },
    };
    
    match ConversationRepository::add_conversation_member(conversation_id, &add_request.node_address) {
        Ok(_) => ApiResponse::Success {
            message: "Member added to conversation".to_string(),
        },
        Err(e) => ApiResponse::Error {
            code: 500,
            message: format!("Failed to add member: {}", e),
        },
    }
}

fn handle_mark_message_read(message_id: i64) -> ApiResponse {
    match MessageRepository::mark_message_read(message_id) {
        Ok(_) => ApiResponse::Success {
            message: "Message marked as read".to_string(),
        },
        Err(e) => ApiResponse::Error {
            code: 500,
            message: format!("Failed to mark message as read: {}", e),
        },
    }
}

// Helper functions for path parameter extraction
fn extract_id(path: &str, prefix: &str) -> Result<i64> {
    let id_str = path.strip_prefix(prefix).ok_or_else(|| anyhow!("Invalid path"))?;
    id_str.parse::<i64>().map_err(|_| anyhow!("Invalid ID"))
}

fn extract_id_from_middle(path: &str, prefix: &str, suffix: &str) -> Result<&str> {
    let without_prefix = path.strip_prefix(prefix).ok_or_else(|| anyhow!("Invalid path"))?;
    without_prefix.strip_suffix(suffix).ok_or_else(|| anyhow!("Invalid path"))
}