// Module imports
use std::collections::HashMap;
use hyperware_process_lib::http::server::{HttpServer, WsMessageType, send_ws_push};
use hyperware_process_lib::LazyLoadBlob;
use hyperware_process_lib::logging::{info, error};
use shared_types::{AppState, WebSocketMessage};

pub use crate::message_handlers::handle_hyperware::{
    handle_http_server_request,
    handle_ws_server_request,
    handle_timer_message,
    handle_terminal_message,
    handle_internal_message,
    handle_external_message,
    make_http_address,
    make_ws_address,
    make_timer_address,
    make_terminal_address,
};

mod handle_hyperware;
pub mod handle_http;
pub mod handle_ws;

// Helper function to get WebSocket channels for a given user
pub fn get_user_channels(state: &AppState, node_address: &str) -> Vec<u32> {
    let mut channels = Vec::new();
    for (&channel_id, user_addr) in &state.connected_users {
        if user_addr == node_address {
            channels.push(channel_id);
        }
    }
    channels
}

// Helper function to broadcast a message to all connected WebSocket clients
pub fn broadcast_to_websocket_clients(
    server: &mut HttpServer,
    state: &AppState,
    message: &WebSocketMessage,
) -> anyhow::Result<()> {
    // Serialize the message
    let ws_message_json = serde_json::to_string(message)?;
    
    // Send to all connected clients
    for &channel_id in state.connected_clients.keys() {
        send_ws_push(
            channel_id,
            WsMessageType::Text,
            LazyLoadBlob {
                mime: Some("application/json".to_string()),
                bytes: ws_message_json.as_bytes().to_vec(),
            },
        );
    }
    
    Ok(())
}