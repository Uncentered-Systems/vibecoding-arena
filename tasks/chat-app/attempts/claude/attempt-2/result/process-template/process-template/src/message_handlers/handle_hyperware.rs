use std::collections::HashMap;
use hyperware_process_lib::{
    Address, Response,
    http::server::{HttpServer, WsMessageType, send_ws_push},
    logging::{info, error},
    LazyLoadBlob,
};
use serde_json;
use shared_types::{MessageChannel, MessageType, AppState};
use crate::log_message;

pub fn make_http_address(our: &Address) -> Address {
    Address::from((our.node(), "http-server", "distro", "sys"))
}

pub fn make_ws_address(our: &Address) -> Address {
    Address::from((our.node(), "ws-server", "distro", "sys"))
}

pub fn make_timer_address(our: &Address) -> Address {
    Address::from((our.node(), "timer", "distro", "sys"))
}

pub fn make_terminal_address(our: &Address) -> Address {
    Address::from((our.node(), "terminal", "distro", "sys"))
}

// HTTP request handler
pub fn handle_http_server_request(
    _our: &Address,
    body: &[u8],
    state: &mut AppState,
    _server: &mut HttpServer,
) -> anyhow::Result<()> {
    // First 4 bytes are the channel ID
    let channel_id = if body.len() >= 4 {
        u32::from_le_bytes([body[0], body[1], body[2], body[3]])
    } else {
        info!("Invalid HTTP request body (too short)");
        return Ok(());
    };
    
    // Log the HTTP request with path info if available
    let content = if body.len() > 4 {
        if let Ok(request_str) = std::str::from_utf8(&body[4..]) {
            Some(format!("HTTP Request: {}", request_str))
        } else {
            Some(format!("Binary HTTP data: {} bytes", body.len() - 4))
        }
    } else {
        Some("Empty HTTP request".to_string())
    };
    
    log_message(
        state,
        "HTTP".to_string(),
        MessageChannel::HttpApi,
        MessageType::HttpGet,
        content,
    );
    
    info!("Received HTTP request");
    
    // Create a simple status response
    let response = serde_json::json!({
        "status": "ok",
        "message": "Hello from Hyperchat!",
        "connected_clients": state.connected_clients.len(),
        "timestamp": crate::get_timestamp()
    });
    
    // Send response
    let response_json = serde_json::to_string(&response)?;
    
    Response::new()
        .body(response_json.as_bytes().to_vec())
        .send()?;
    
    Ok(())
}

// WebSocket handler
pub fn handle_ws_server_request(
    _our: &Address,
    body: &[u8],
    state: &mut AppState,
    _server: &mut HttpServer,
) -> anyhow::Result<()> {
    info!("Received WebSocket event");
    
    
    // First 4 bytes are the channel ID
    let channel_id = u32::from_le_bytes([body[0], body[1], body[2], body[3]]);
    
    // Next byte indicates the event type: 0 = open, 1 = message, 2 = close
    let event_type = body[4];
    
    match event_type {
        0 => { // Open event
            // Store client information
            state.connected_clients.insert(channel_id, "/ws".to_string());
            
            // Log connection
            log_message(
                state,
                "WebSocket".to_string(),
                MessageChannel::WebSocket,
                MessageType::WebSocketOpen,
                Some(format!("WebSocket connection opened: {}", channel_id)),
            );
            
            // Send welcome message
            let welcome = serde_json::json!({
                "type": "welcome",
                "message": "Welcome to Hyperchat!",
                "channel_id": channel_id
            });
            
            if let Ok(welcome_json) = serde_json::to_string(&welcome) {
                send_ws_push(
                    channel_id,
                    WsMessageType::Text,
                    LazyLoadBlob {
                        mime: Some("application/json".to_string()),
                        bytes: welcome_json.as_bytes().to_vec(),
                    },
                );
            }
        },
        1 => { // Message event
            // Extract message content starting from byte 5
            if body.len() > 5 {
                let message = &body[5..];
                
                if let Ok(message_str) = std::str::from_utf8(message) {
                    // Log message
                    log_message(
                        state,
                        "WebSocket".to_string(),
                        MessageChannel::WebSocket,
                        MessageType::WebSocketPushA,
                        Some(format!("WebSocket message: {}", message_str)),
                    );
                    
                    // Echo message back
                    let response = serde_json::json!({
                        "type": "echo",
                        "original": message_str,
                        "timestamp": crate::get_timestamp()
                    });
                    
                    if let Ok(response_json) = serde_json::to_string(&response) {
                        send_ws_push(
                            channel_id,
                            WsMessageType::Text,
                            LazyLoadBlob {
                                mime: Some("application/json".to_string()),
                                bytes: response_json.as_bytes().to_vec(),
                            },
                        );
                    }
                }
            }
        },
        2 => { // Close event
            // Remove client
            state.connected_clients.remove(&channel_id);
            
            // Log disconnection
            log_message(
                state,
                "WebSocket".to_string(),
                MessageChannel::WebSocket,
                MessageType::WebSocketClose,
                Some(format!("WebSocket connection closed: {}", channel_id)),
            );
        },
        _ => {
            error!("Unknown WebSocket event type: {}", event_type);
        }
    }
    
    Ok(())
}

// Timer handler, usually used for time-based events
pub fn handle_timer_message(
    _body: &[u8],
    state: &mut AppState,
    _server: &mut HttpServer,
) -> anyhow::Result<()> {
    // Log the timer message
    log_message(
        state,
        "Timer".to_string(),
        MessageChannel::Timer,
        MessageType::TimerTick,
        Some("Timer event received".to_string()),
    );
    info!("Received timer message");
    
    Ok(())
}

// Terminal handler for debugging 
pub fn handle_terminal_message(
    body: &[u8],
    state: &mut AppState,
    _server: &mut HttpServer,
) -> anyhow::Result<()> {
    // Log the terminal message
    let content = if let Ok(str) = std::str::from_utf8(body) {
        Some(str.to_string())
    } else {
        Some(format!("Binary data: {} bytes", body.len()))
    };
    
    log_message(
        state,
        "Terminal".to_string(),
        MessageChannel::Terminal,
        MessageType::TerminalCommand,
        content,
    );
    
    Ok(())
}

pub fn handle_internal_message(
    source: &Address,
    body: &[u8],
    state: &mut AppState,
    _server: &mut HttpServer,
) -> anyhow::Result<()> {
    // Log the internal message
    let content = if let Ok(str) = std::str::from_utf8(body) {
        Some(str.to_string())
    } else {
        Some(format!("Binary data: {} bytes", body.len()))
    };
    
    log_message(
        state,
        format!("Internal:{}", source),
        MessageChannel::Internal,
        MessageType::LocalRequest,
        content,
    );
    
    // Simple response for internal messages
    let response = serde_json::json!({
        "status": "ok",
        "message": "Message received",
        "message_count": state.message_history.len()
    });

    Response::new()
        .body(serde_json::to_vec(&response)?)
        .send()?;
    
    Ok(())
}

pub fn handle_external_message(
    source: &Address,
    body: &[u8],
    state: &mut AppState,
    _server: &mut HttpServer,
) -> anyhow::Result<()> {
    // Parse the message
    let message_str = match std::str::from_utf8(body) {
        Ok(s) => s,
        Err(_) => {
            info!("Received binary message: {} bytes", body.len());
            // Log binary message
            log_message(
                state,
                format!("External:{}", source),
                MessageChannel::External,
                MessageType::RemoteRequest,
                Some(format!("Binary message: {} bytes", body.len())),
            );
            
            // Send response
            let response = serde_json::json!({
                "status": "error",
                "message": "Binary messages not supported"
            });
            
            Response::new()
                .body(serde_json::to_vec(&response)?)
                .send()?;
                
            return Ok(());
        }
    };
    
    // Log the external message
    info!("Received external message: {}", message_str);
    log_message(
        state,
        format!("External message from {}", source),
        MessageChannel::External,
        MessageType::RemoteRequest,
        Some(message_str.to_string()),
    );
    
    // Parse JSON if possible
    let parsed_json = serde_json::from_str::<serde_json::Value>(message_str);
    
    // Respond based on content
    let response = match parsed_json {
        Ok(json) => {
            // Successfully parsed as JSON
            serde_json::json!({
                "status": "ok",
                "message": "JSON message received",
                "echo": json,
                "timestamp": crate::get_timestamp()
            })
        },
        Err(_) => {
            // Plain text message
            serde_json::json!({
                "status": "ok",
                "message": "Text message received",
                "echo": message_str,
                "timestamp": crate::get_timestamp()
            })
        }
    };
    
    // Send response
    Response::new()
        .body(serde_json::to_vec(&response)?)
        .send()?;
    
    Ok(())
}