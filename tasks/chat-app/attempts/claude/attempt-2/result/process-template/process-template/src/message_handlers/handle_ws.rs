use anyhow::Result;
use hyperware_process_lib::{
    http::server::{HttpServer, WsRequest, WsEventData, WsMessageType, send_ws_push},
    logging::{error, info},
    Address, LazyLoadBlob
};
use serde_json;
use shared_types::{
    AppState, MessageChannel, MessageType, WebSocketMessage
};

use crate::database::{ContactRepository, get_timestamp};
use crate::log_message;

pub fn handle_ws_message(
    _our: &Address,
    body: &[u8],
    state: &mut AppState,
    server: &mut HttpServer,
) -> Result<()> {
    let ws_req = serde_json::from_slice::<WsRequest>(body)?;
    
    match ws_req.event {
        WsEventData::Open => {
            // Store the connection in our state
            state.connected_clients.insert(ws_req.channel_id, "/ws".to_string());
            
            info!("WebSocket connection opened: {}", ws_req.channel_id);
            
            // Log the connection
            log_message(
                state,
                "WebSocket".to_string(),
                MessageChannel::WebSocket,
                MessageType::WebSocketOpen,
                Some(format!("WebSocket connection opened: {}", ws_req.channel_id)),
            );
            
            // Send welcome message
            let welcome = serde_json::json!({
                "type": "welcome",
                "message": "Welcome to Hyperchat WebSocket server",
                "channel_id": ws_req.channel_id
            });
            
            if let Ok(welcome_json) = serde_json::to_string(&welcome) {
                send_ws_push(
                    ws_req.channel_id,
                    WsMessageType::Text,
                    LazyLoadBlob {
                        mime: Some("application/json".to_string()),
                        bytes: welcome_json.as_bytes().to_vec(),
                    },
                );
            }
        },
        WsEventData::Close => {
            // Remove from connected clients
            state.connected_clients.remove(&ws_req.channel_id);
            
            // If this user had authenticated, update their status and remove mapping
            if let Some(node_address) = state.connected_users.remove(&ws_req.channel_id) {
                info!("User disconnected: {}", node_address);
                
                // Update contact status to offline
                if let Err(e) = ContactRepository::update_contact_status(&node_address, "offline") {
                    error!("Failed to update contact status: {}", e);
                }
            }
            
            info!("WebSocket connection closed: {}", ws_req.channel_id);
            
            // Log the disconnection
            log_message(
                state,
                "WebSocket".to_string(),
                MessageChannel::WebSocket,
                MessageType::WebSocketClose,
                Some(format!("WebSocket connection closed: {}", ws_req.channel_id)),
            );
        },
        WsEventData::Push(ref message) => {
            info!("Received WebSocket message: {}", message);
            
            // Log the message
            log_message(
                state,
                "WebSocket".to_string(),
                MessageChannel::WebSocket,
                MessageType::WebSocketPushA,
                Some(format!("WebSocket message: {}", message)),
            );
            
            // Process WebSocket messages
            if let Ok(ws_message) = serde_json::from_str::<WebSocketMessage>(message) {
                match ws_message {
                    WebSocketMessage::Auth { node_address } => {
                        // Handle authentication
                        state.connected_users.insert(ws_req.channel_id, node_address.clone());
                        
                        info!("User authenticated: {}", node_address);
                        
                        // Update contact status to online
                        if let Err(e) = ContactRepository::update_contact_status(&node_address, "online") {
                            error!("Failed to update contact status: {}", e);
                        }
                        
                        // Send acknowledgement
                        let ack = serde_json::json!({
                            "type": "auth_success",
                            "node_address": node_address,
                            "timestamp": get_timestamp()
                        });
                        
                        if let Ok(ack_json) = serde_json::to_string(&ack) {
                            send_ws_push(
                                ws_req.channel_id,
                                WsMessageType::Text,
                                LazyLoadBlob {
                                    mime: Some("application/json".to_string()),
                                    bytes: ack_json.as_bytes().to_vec(),
                                },
                            );
                        }
                        
                        // Notify other users that this user is online
                        let status_change = serde_json::json!({
                            "type": "status_change",
                            "node_address": node_address,
                            "status": "online",
                            "timestamp": get_timestamp()
                        });
                        
                        if let Ok(status_json) = serde_json::to_string(&status_change) {
                            for (&client_id, _) in &state.connected_clients {
                                if client_id != ws_req.channel_id {
                                    send_ws_push(
                                        client_id,
                                        WsMessageType::Text,
                                        LazyLoadBlob {
                                            mime: Some("application/json".to_string()),
                                            bytes: status_json.as_bytes().to_vec(),
                                        },
                                    );
                                }
                            }
                        }
                    },
                    WebSocketMessage::TypingIndicator { conversation_id, user_id, is_typing } => {
                        // Broadcast typing indicator to all members of the conversation
                        let typing_indicator = serde_json::json!({
                            "type": "typing_indicator",
                            "conversation_id": conversation_id,
                            "user_id": user_id,
                            "is_typing": is_typing,
                            "timestamp": get_timestamp()
                        });
                        
                        if let Ok(typing_json) = serde_json::to_string(&typing_indicator) {
                            for (&client_id, _) in &state.connected_clients {
                                if client_id != ws_req.channel_id {
                                    send_ws_push(
                                        client_id,
                                        WsMessageType::Text,
                                        LazyLoadBlob {
                                            mime: Some("application/json".to_string()),
                                            bytes: typing_json.as_bytes().to_vec(),
                                        },
                                    );
                                }
                            }
                        }
                    },
                    WebSocketMessage::ReadReceipt { message_ids } => {
                        // Broadcast read receipt to all clients
                        let read_receipt = serde_json::json!({
                            "type": "read_receipt",
                            "message_ids": message_ids,
                            "timestamp": get_timestamp()
                        });
                        
                        if let Ok(receipt_json) = serde_json::to_string(&read_receipt) {
                            for (&client_id, _) in &state.connected_clients {
                                if client_id != ws_req.channel_id {
                                    send_ws_push(
                                        client_id,
                                        WsMessageType::Text,
                                        LazyLoadBlob {
                                            mime: Some("application/json".to_string()),
                                            bytes: receipt_json.as_bytes().to_vec(),
                                        },
                                    );
                                }
                            }
                        }
                    },
                    // Messages are handled through HTTP API for persistence
                    WebSocketMessage::ChatMessage { .. } => {
                        let error_message = serde_json::json!({
                            "type": "error",
                            "message": "Chat messages should be sent via HTTP API for persistence",
                            "timestamp": get_timestamp()
                        });
                        
                        if let Ok(error_json) = serde_json::to_string(&error_message) {
                            send_ws_push(
                                ws_req.channel_id,
                                WsMessageType::Text,
                                LazyLoadBlob {
                                    mime: Some("application/json".to_string()),
                                    bytes: error_json.as_bytes().to_vec(),
                                },
                            );
                        }
                    },
                    WebSocketMessage::StatusChange { node_address, status } => {
                        // Update contact status
                        if let Err(e) = ContactRepository::update_contact_status(&node_address, &status) {
                            error!("Failed to update contact status: {}", e);
                        }
                        
                        // Broadcast status change
                        let status_change = serde_json::json!({
                            "type": "status_change",
                            "node_address": node_address,
                            "status": status,
                            "timestamp": get_timestamp()
                        });
                        
                        if let Ok(status_json) = serde_json::to_string(&status_change) {
                            for (&client_id, _) in &state.connected_clients {
                                if client_id != ws_req.channel_id {
                                    send_ws_push(
                                        client_id,
                                        WsMessageType::Text,
                                        LazyLoadBlob {
                                            mime: Some("application/json".to_string()),
                                            bytes: status_json.as_bytes().to_vec(),
                                        },
                                    );
                                }
                            }
                        }
                    }
                }
            } else {
                // If we can't parse as WebSocketMessage, respond with error
                let error_message = serde_json::json!({
                    "type": "error",
                    "message": "Invalid message format",
                    "timestamp": get_timestamp()
                });
                
                if let Ok(error_json) = serde_json::to_string(&error_message) {
                    send_ws_push(
                        ws_req.channel_id,
                        WsMessageType::Text,
                        LazyLoadBlob {
                            mime: Some("application/json".to_string()),
                            bytes: error_json.as_bytes().to_vec(),
                        },
                    );
                }
            }
        }
    }
    
    Ok(())
}