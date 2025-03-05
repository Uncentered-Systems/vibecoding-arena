use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::hyperware::process::hyperware_chat::{
    Request as HyperwareChatRequest, Response as HyperwareChatResponse, SendRequest, HyperwareChatMessage,
};
use hyperware_process_lib::kv::{self, Kv};
use hyperware_process_lib::logging::{error, info, init_logging, Level};
use hyperware_process_lib::{
    await_message, call_init, get_blob,
    http::server::{
        send_response, HttpBindingConfig, HttpServer, HttpServerRequest, StatusCode,
        WsBindingConfig, WsMessageType,
    },
    println, Address, LazyLoadBlob, Message as ProcessMessage, Request, Response, PackageId,
};
use serde::{Deserialize, Serialize};

wit_bindgen::generate!({
    path: "target/wit",
    world: "hyperware-chat-template-dot-os-v0",
    generate_unused_types: true,
    additional_derives: [serde::Deserialize, serde::Serialize, process_macros::SerdeJsonInto],
});

const HTTP_API_PATH: &str = "/messages";
const WS_PATH: &str = "/";
const CONTACTS_API_PATH: &str = "/contacts";
const GROUPS_API_PATH: &str = "/groups";

// Database names
const MESSAGES_DB: &str = "messages";
const GROUPS_DB: &str = "groups";
const GROUP_MESSAGES_DB: &str = "group_messages";

#[derive(Debug, Clone, Serialize, Deserialize, process_macros::SerdeJsonInto)]
struct NewMessage {
    hyperware_chat: String,
    author: String,
    content: String,
    timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessage {
    author: String,
    content: String,
    timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, process_macros::SerdeJsonInto)]
struct Group {
    id: String,
    name: String,
    members: HashSet<String>,
    created_by: String,
    created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GroupMessage {
    group_id: String,
    author: String,
    content: String,
    timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, process_macros::SerdeJsonInto)]
struct CreateGroupRequest {
    name: String,
    members: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, process_macros::SerdeJsonInto)]
struct GroupMemberRequest {
    group_id: String,
    member: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, process_macros::SerdeJsonInto)]
struct SendGroupMessageRequest {
    group_id: String,
    message: String,
}

// Wrapper for HTTP responses
#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

type MessageArchive = HashMap<String, Vec<ChatMessage>>;

// Used to store contacts info
struct ChatStore {
    package_id: PackageId,
    messages_db: Kv<String, Vec<ChatMessage>>,
    groups_db: Kv<String, Group>,
    group_messages_db: Kv<String, Vec<GroupMessage>>,
}

impl ChatStore {
    fn new(package_id: PackageId) -> anyhow::Result<Self> {
        let messages_db = kv::open(package_id.clone(), MESSAGES_DB, None)?;
        let groups_db = kv::open(package_id.clone(), GROUPS_DB, None)?;
        let group_messages_db = kv::open(package_id.clone(), GROUP_MESSAGES_DB, None)?;

        Ok(Self {
            package_id,
            messages_db,
            groups_db,
            group_messages_db,
        })
    }

    // Message methods
    fn get_messages(&self, contact: &str) -> anyhow::Result<Vec<ChatMessage>> {
        match self.messages_db.get(&contact.to_string()) {
            Ok(messages) => Ok(messages),
            Err(_) => Ok(Vec::new()),
        }
    }

    fn add_message(&self, contact: &str, message: ChatMessage) -> anyhow::Result<()> {
        let mut messages = self.get_messages(contact)?;
        messages.push(message);
        self.messages_db.set(&contact.to_string(), &messages, None)?;
        Ok(())
    }

    fn get_all_messages(&self) -> anyhow::Result<MessageArchive> {
        // For a real implementation, we would need to list keys from KV store
        // This is a simplified version that will only return messages we've interacted with
        // in the current session
        Ok(HashMap::new())
    }

    // Group methods
    fn create_group(&self, name: &str, members: HashSet<String>, created_by: &str) -> anyhow::Result<Group> {
        let id = format!("group_{}", SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis());
        let group = Group {
            id: id.clone(),
            name: name.to_string(),
            members,
            created_by: created_by.to_string(),
            created_at: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        };
        
        self.groups_db.set(&id, &group, None)?;
        Ok(group)
    }

    fn get_group(&self, group_id: &str) -> anyhow::Result<Option<Group>> {
        match self.groups_db.get(&group_id.to_string()) {
            Ok(group) => Ok(Some(group)),
            Err(_) => Ok(None),
        }
    }

    fn add_member_to_group(&self, group_id: &str, member: &str) -> anyhow::Result<bool> {
        if let Some(mut group) = self.get_group(group_id)? {
            let was_added = group.members.insert(member.to_string());
            if was_added {
                self.groups_db.set(&group_id.to_string(), &group, None)?;
            }
            Ok(was_added)
        } else {
            Ok(false)
        }
    }

    fn remove_member_from_group(&self, group_id: &str, member: &str) -> anyhow::Result<bool> {
        if let Some(mut group) = self.get_group(group_id)? {
            let was_removed = group.members.remove(member);
            if was_removed {
                self.groups_db.set(&group_id.to_string(), &group, None)?;
            }
            Ok(was_removed)
        } else {
            Ok(false)
        }
    }

    fn get_all_groups(&self) -> anyhow::Result<Vec<Group>> {
        // For a real implementation, we would need to list keys from KV store
        // This is a simplified version
        let groups = Vec::new();
        
        // We'd normally iterate over all keys, for now let's return an empty list
        // as we can't enumerate keys in KV store without additional tracking

        Ok(groups)
    }

    // Group message methods
    fn add_group_message(&self, group_id: &str, message: GroupMessage) -> anyhow::Result<()> {
        let mut messages = match self.group_messages_db.get(&group_id.to_string()) {
            Ok(messages) => messages,
            Err(_) => Vec::new(),
        };
        
        messages.push(message);
        self.group_messages_db.set(&group_id.to_string(), &messages, None)?;
        Ok(())
    }

    fn get_group_messages(&self, group_id: &str) -> anyhow::Result<Vec<GroupMessage>> {
        match self.group_messages_db.get(&group_id.to_string()) {
            Ok(messages) => Ok(messages),
            Err(_) => Ok(Vec::new()),
        }
    }
}

fn make_http_address(our: &Address) -> Address {
    Address::from((our.node(), "http_server", "distro", "sys"))
}

fn make_contacts_address(our: &Address) -> Address {
    Address::from((our.node(), "contacts", "contacts", "sys"))
}

// Get timestamp in seconds
fn get_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn handle_http_server_request(
    body: &[u8],
    request: HttpServerRequest,
    store: &ChatStore,
    server: &mut HttpServer,
) -> anyhow::Result<()> {
    match request {
        HttpServerRequest::WebSocketOpen {
            ref path,
            channel_id,
        } => server.handle_websocket_open(path, channel_id),
        HttpServerRequest::WebSocketClose(channel_id) => server.handle_websocket_close(channel_id),
        HttpServerRequest::WebSocketPush { .. } => {
            let Some(blob) = get_blob() else {
                return Ok(());
            };

            handle_chat_request(
                our,
                &make_http_address(our),
                &blob.bytes,
                true,
                store,
                server,
            )?;
        }
        HttpServerRequest::Http(request) => {
            let path = request.bound_path(None);
            let method_str = request.method().unwrap();
            let method = method_str.as_str();
            
            // Handle messages API
            if path.starts_with(HTTP_API_PATH) {
                match method {
                    // Get all messages
                    "GET" => {
                        let headers = HashMap::from([(
                            "Content-Type".to_string(),
                            "application/json".to_string(),
                        )]);

                        let messages = store.get_all_messages().unwrap_or_default();
                        
                        send_response(
                            StatusCode::OK,
                            Some(headers),
                            serde_json::to_vec(&serde_json::json!({
                                "History": {
                                    "messages": messages
                                }
                            }))
                            .unwrap(),
                        );
                    }
                    // Send a message
                    "POST" => {
                        let Some(blob) = get_blob() else {
                            send_response(StatusCode::BAD_REQUEST, None, vec![]);
                            return Ok(());
                        };
                        
                        handle_chat_request(
                            our,
                            &make_http_address(our),
                            &blob.bytes,
                            true,
                            store,
                            server,
                        )?;

                        send_response(StatusCode::CREATED, None, vec![]);
                    }
                    _ => send_response(StatusCode::METHOD_NOT_ALLOWED, None, vec![]),
                }
            }
            // Handle groups API
            else if path.starts_with(GROUPS_API_PATH) {
                let headers = HashMap::from([(
                    "Content-Type".to_string(),
                    "application/json".to_string(),
                )]);

                match method {
                    "GET" => {
                        // Get a specific group or list all groups
                        let query_params = request.query_params();
                        if let Some(group_id) = query_params.get("id") {
                            // Get specific group and its messages
                            if let Ok(Some(group)) = store.get_group(group_id) {
                                let messages = store.get_group_messages(group_id).unwrap_or_default();
                                
                                send_response(
                                    StatusCode::OK,
                                    Some(headers),
                                    serde_json::to_vec(&ApiResponse {
                                        success: true,
                                        data: Some(serde_json::json!({
                                            "group": group,
                                            "messages": messages
                                        })),
                                        error: None,
                                    })
                                    .unwrap(),
                                );
                            } else {
                                send_response(
                                    StatusCode::NOT_FOUND,
                                    Some(headers),
                                    serde_json::to_vec(&ApiResponse::<()> {
                                        success: false,
                                        data: None,
                                        error: Some("Group not found".to_string()),
                                    })
                                    .unwrap(),
                                );
                            }
                        } else {
                            // List all groups
                            let groups = store.get_all_groups().unwrap_or_default();
                            
                            send_response(
                                StatusCode::OK,
                                Some(headers),
                                serde_json::to_vec(&ApiResponse {
                                    success: true,
                                    data: Some(serde_json::json!({ "groups": groups })),
                                    error: None,
                                })
                                .unwrap(),
                            );
                        }
                    }
                    "POST" => {
                        // Create a new group
                        let Some(blob) = get_blob() else {
                            send_response(StatusCode::BAD_REQUEST, None, vec![]);
                            return Ok(());
                        };
                        
                        // Parse create group request
                        if let Ok(create_group_req) = serde_json::from_slice::<CreateGroupRequest>(&blob.bytes) {
                            let members: HashSet<String> = create_group_req.members.into_iter().collect();
                            
                            // Add the creator to the group members
                            let mut all_members = members.clone();
                            all_members.insert(our.node.clone());
                            
                            match store.create_group(&create_group_req.name, all_members, &our.node) {
                                Ok(group) => {
                                    send_response(
                                        StatusCode::CREATED,
                                        Some(headers),
                                        serde_json::to_vec(&ApiResponse {
                                            success: true,
                                            data: Some(serde_json::json!({ "group": group })),
                                            error: None,
                                        })
                                        .unwrap(),
                                    );
                                }
                                Err(e) => {
                                    send_response(
                                        StatusCode::INTERNAL_SERVER_ERROR,
                                        Some(headers),
                                        serde_json::to_vec(&ApiResponse::<()> {
                                            success: false,
                                            data: None,
                                            error: Some(format!("Failed to create group: {}", e)),
                                        })
                                        .unwrap(),
                                    );
                                }
                            }
                        } else {
                            send_response(
                                StatusCode::BAD_REQUEST,
                                Some(headers),
                                serde_json::to_vec(&ApiResponse::<()> {
                                    success: false,
                                    data: None,
                                    error: Some("Invalid group data".to_string()),
                                })
                                .unwrap(),
                            );
                        }
                    }
                    "PUT" => {
                        // Add/remove members or send message to group
                        let Some(blob) = get_blob() else {
                            send_response(StatusCode::BAD_REQUEST, None, vec![]);
                            return Ok(());
                        };
                        
                        let query_params = request.query_params();
                        let empty_string = "".to_string();
                        let action = query_params.get("action").unwrap_or(&empty_string);
                        
                        match action.as_str() {
                            "add_member" => {
                                if let Ok(member_req) = serde_json::from_slice::<GroupMemberRequest>(&blob.bytes) {
                                    match store.add_member_to_group(&member_req.group_id, &member_req.member) {
                                        Ok(true) => {
                                            send_response(
                                                StatusCode::OK,
                                                Some(headers),
                                                serde_json::to_vec(&ApiResponse::<()> {
                                                    success: true,
                                                    data: None,
                                                    error: None,
                                                })
                                                .unwrap(),
                                            );
                                        }
                                        Ok(false) => {
                                            send_response(
                                                StatusCode::NOT_FOUND,
                                                Some(headers),
                                                serde_json::to_vec(&ApiResponse::<()> {
                                                    success: false,
                                                    data: None,
                                                    error: Some("Group not found or member already in group".to_string()),
                                                })
                                                .unwrap(),
                                            );
                                        }
                                        Err(e) => {
                                            send_response(
                                                StatusCode::INTERNAL_SERVER_ERROR,
                                                Some(headers),
                                                serde_json::to_vec(&ApiResponse::<()> {
                                                    success: false,
                                                    data: None,
                                                    error: Some(format!("Failed to add member: {}", e)),
                                                })
                                                .unwrap(),
                                            );
                                        }
                                    }
                                } else {
                                    send_response(
                                        StatusCode::BAD_REQUEST,
                                        Some(headers),
                                        serde_json::to_vec(&ApiResponse::<()> {
                                            success: false,
                                            data: None,
                                            error: Some("Invalid member data".to_string()),
                                        })
                                        .unwrap(),
                                    );
                                }
                            }
                            "remove_member" => {
                                if let Ok(member_req) = serde_json::from_slice::<GroupMemberRequest>(&blob.bytes) {
                                    match store.remove_member_from_group(&member_req.group_id, &member_req.member) {
                                        Ok(true) => {
                                            send_response(
                                                StatusCode::OK,
                                                Some(headers),
                                                serde_json::to_vec(&ApiResponse::<()> {
                                                    success: true,
                                                    data: None,
                                                    error: None,
                                                })
                                                .unwrap(),
                                            );
                                        }
                                        Ok(false) => {
                                            send_response(
                                                StatusCode::NOT_FOUND,
                                                Some(headers),
                                                serde_json::to_vec(&ApiResponse::<()> {
                                                    success: false,
                                                    data: None,
                                                    error: Some("Group not found or member not in group".to_string()),
                                                })
                                                .unwrap(),
                                            );
                                        }
                                        Err(e) => {
                                            send_response(
                                                StatusCode::INTERNAL_SERVER_ERROR,
                                                Some(headers),
                                                serde_json::to_vec(&ApiResponse::<()> {
                                                    success: false,
                                                    data: None,
                                                    error: Some(format!("Failed to remove member: {}", e)),
                                                })
                                                .unwrap(),
                                            );
                                        }
                                    }
                                } else {
                                    send_response(
                                        StatusCode::BAD_REQUEST,
                                        Some(headers),
                                        serde_json::to_vec(&ApiResponse::<()> {
                                            success: false,
                                            data: None,
                                            error: Some("Invalid member data".to_string()),
                                        })
                                        .unwrap(),
                                    );
                                }
                            }
                            "send_message" => {
                                if let Ok(msg_req) = serde_json::from_slice::<SendGroupMessageRequest>(&blob.bytes) {
                                    // Check if group exists and user is a member
                                    if let Ok(Some(group)) = store.get_group(&msg_req.group_id) {
                                        if !group.members.contains(&our.node) {
                                            send_response(
                                                StatusCode::FORBIDDEN,
                                                Some(headers),
                                                serde_json::to_vec(&ApiResponse::<()> {
                                                    success: false,
                                                    data: None,
                                                    error: Some("You are not a member of this group".to_string()),
                                                })
                                                .unwrap(),
                                            );
                                            return Ok(());
                                        }
                                        
                                        // Add message to group
                                        let group_message = GroupMessage {
                                            group_id: msg_req.group_id.clone(),
                                            author: our.node.clone(),
                                            content: msg_req.message.clone(),
                                            timestamp: get_timestamp(),
                                        };
                                        
                                        match store.add_group_message(&msg_req.group_id, group_message.clone()) {
                                            Ok(_) => {
                                                // Notify all group members
                                                for member in &group.members {
                                                    if member != &our.node {
                                                        // Send message to member
                                                        // In a real implementation, we would need to implement this
                                                    }
                                                }
                                                
                                                // Send WebSocket message to update UI
                                                let blob = LazyLoadBlob {
                                                    mime: Some("application/json".to_string()),
                                                    bytes: serde_json::to_vec(&serde_json::json!({
                                                        "NewGroupMessage": {
                                                            "group_id": msg_req.group_id,
                                                            "author": our.node,
                                                            "content": msg_req.message,
                                                            "timestamp": group_message.timestamp,
                                                        }
                                                    }))
                                                    .unwrap(),
                                                };
                                                server.ws_push_all_channels(WS_PATH, WsMessageType::Text, blob);
                                                
                                                send_response(
                                                    StatusCode::OK,
                                                    Some(headers),
                                                    serde_json::to_vec(&ApiResponse::<()> {
                                                        success: true,
                                                        data: None,
                                                        error: None,
                                                    })
                                                    .unwrap(),
                                                );
                                            }
                                            Err(e) => {
                                                send_response(
                                                    StatusCode::INTERNAL_SERVER_ERROR,
                                                    Some(headers),
                                                    serde_json::to_vec(&ApiResponse::<()> {
                                                        success: false,
                                                        data: None,
                                                        error: Some(format!("Failed to send message: {}", e)),
                                                    })
                                                    .unwrap(),
                                                );
                                            }
                                        }
                                    } else {
                                        send_response(
                                            StatusCode::NOT_FOUND,
                                            Some(headers),
                                            serde_json::to_vec(&ApiResponse::<()> {
                                                success: false,
                                                data: None,
                                                error: Some("Group not found".to_string()),
                                            })
                                            .unwrap(),
                                        );
                                    }
                                } else {
                                    send_response(
                                        StatusCode::BAD_REQUEST,
                                        Some(headers),
                                        serde_json::to_vec(&ApiResponse::<()> {
                                            success: false,
                                            data: None,
                                            error: Some("Invalid message data".to_string()),
                                        })
                                        .unwrap(),
                                    );
                                }
                            }
                            _ => {
                                send_response(
                                    StatusCode::BAD_REQUEST,
                                    Some(headers),
                                    serde_json::to_vec(&ApiResponse::<()> {
                                        success: false,
                                        data: None,
                                        error: Some("Invalid action".to_string()),
                                    })
                                    .unwrap(),
                                );
                            }
                        }
                    }
                    "DELETE" => {
                        // Delete a group (not implemented for simplicity)
                        send_response(
                            StatusCode::NOT_IMPLEMENTED,
                            Some(headers),
                            serde_json::to_vec(&ApiResponse::<()> {
                                success: false,
                                data: None,
                                error: Some("Not implemented".to_string()),
                            })
                            .unwrap(),
                        );
                    }
                    _ => send_response(StatusCode::METHOD_NOT_ALLOWED, None, vec![]),
                }
            }
            // Handle contacts API - proxy to the system contacts process
            else if path.starts_with(CONTACTS_API_PATH) {
                // This would forward requests to the contacts system process
                // For simplicity, we're not implementing this fully
                let headers = HashMap::from([(
                    "Content-Type".to_string(),
                    "application/json".to_string(),
                )]);
                
                // Just return an empty contacts list for now
                send_response(
                    StatusCode::OK,
                    Some(headers),
                    serde_json::to_vec(&ApiResponse {
                        success: true,
                        data: Some(serde_json::json!({ "contacts": [] })),
                        error: None,
                    })
                    .unwrap(),
                );
            }
            else {
                send_response(StatusCode::NOT_FOUND, None, vec![]);
            }
        }
    };

    Ok(())
}

fn handle_chat_request(
    our: &Address,
    source: &Address,
    body: &[u8],
    is_http: bool,
    store: &ChatStore,
    server: &HttpServer,
) -> anyhow::Result<()> {
    match body.try_into()? {
        HyperwareChatRequest::Send(SendRequest {
            ref target,
            ref message,
        }) => {
            // Counterparty is the other node in the hyperware-chat with us
            let (counterparty, author) = if target == &our.node {
                (&source.node, source.node.clone())
            } else {
                (target, our.node.clone())
            };

            // If the target is not us, send a request to the target
            if target == &our.node {
                println!("{}: {}", source.node, message);
            } else {
                Request::new()
                    .target((target, "hyperware-chat", "hyperware-chat", "template.os"))
                    .body(body)
                    .send_and_await_response(5)??;
            }

            // Insert message into archive
            let new_message = ChatMessage {
                author: author.clone(),
                content: message.clone(),
                timestamp: get_timestamp(),
            };
            
            // Add message to store
            store.add_message(counterparty, new_message.clone())?;

            if is_http {
                // If is HTTP from FE: done
                return Ok(());
            }

            // Not HTTP from FE: send response to node & update any FE listeners
            Response::new().body(HyperwareChatResponse::Send).send()?;

            // Send a WebSocket message to the http server in order to update the UI
            let blob = LazyLoadBlob {
                mime: Some("application/json".to_string()),
                bytes: serde_json::to_vec(&serde_json::json!({
                    "NewMessage": NewMessage {
                        hyperware_chat: counterparty.to_string(),
                        author,
                        content: message.to_string(),
                        timestamp: new_message.timestamp,
                    }
                }))
                .unwrap(),
            };
            server.ws_push_all_channels(WS_PATH, WsMessageType::Text, blob);
        }
        HyperwareChatRequest::History(ref node) => {
            let messages = store.get_messages(node)?;
            
            // Convert to WIT message format
            let wit_messages: Vec<HyperwareChatMessage> = messages
                .iter()
                .map(|msg| HyperwareChatMessage {
                    author: msg.author.clone(),
                    content: msg.content.clone(),
                })
                .collect();
            
            Response::new()
                .body(HyperwareChatResponse::History(wit_messages))
                .send()?;
        }
    }
    Ok(())
}

fn handle_message(
    our: &Address,
    message: &ProcessMessage,
    store: &ChatStore,
    server: &mut HttpServer,
) -> anyhow::Result<()> {
    if !message.is_request() {
        return Ok(());
    }

    let body = message.body();
    let source = message.source();

    if source == &make_http_address(our) {
        // Parse HTTP request
        let Ok(request) = serde_json::from_slice::<HttpServerRequest>(body) else {
            // Fail quietly if we can't parse the request
            info!("couldn't parse message from http_server: {body:?}");
            return Ok(());
        };
        
        handle_http_server_request(body, request, store, server)?;
    } else {
        handle_chat_request(our, source, body, false, store, server)?;
    }

    Ok(())
}

call_init!(init);
fn init(our: Address) {
    init_logging(Level::DEBUG, Level::INFO, None, None, None).unwrap();
    info!("begin");

    // Create store for persistent data
    let package_id = our.package_id();
    let store = match ChatStore::new(package_id) {
        Ok(store) => store,
        Err(e) => {
            error!("Failed to create store: {}", e);
            return;
        }
    };

    let mut server = HttpServer::new(5);

    // Bind UI files to routes with index.html at "/"; API to /messages; WS to "/"
    server
        .serve_ui("ui", vec!["/"], HttpBindingConfig::default())
        .expect("failed to serve UI");
    
    // API endpoints
    server
        .bind_http_path(HTTP_API_PATH, HttpBindingConfig::default())
        .expect("failed to bind messages API");
    server
        .bind_http_path(CONTACTS_API_PATH, HttpBindingConfig::default())
        .expect("failed to bind contacts API");
    server
        .bind_http_path(GROUPS_API_PATH, HttpBindingConfig::default())
        .expect("failed to bind groups API");
    
    // WebSocket for real-time updates
    server
        .bind_ws_path(WS_PATH, WsBindingConfig::default())
        .expect("failed to bind WS API");

    loop {
        match await_message() {
            Err(send_error) => error!("got SendError: {send_error}"),
            Ok(ref message) => {
                match handle_message(&our, message, &store, &mut server) {
                    Ok(_) => {}
                    Err(e) => error!("got error while handling message: {e:?}"),
                }
            }
        }
    }
}
