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
    hyperware::process::standard, // Added for our() function
};
use serde::{Deserialize, Serialize};

wit_bindgen::generate!({
    path: "target/wit",
    world: "hyperware-chat-template-dot-os-v0",
    generate_unused_types: true,
    additional_derives: [serde::Deserialize, serde::Serialize, process_macros::SerdeJsonInto],
});

// Paths for API endpoints
const PROCESS_PATH: &str = "hyperware-chat:hyperware-chat:template.os";
const HTTP_API_PATH: &str = "/messages";
const WS_PATH: &str = "/";
const CONTACTS_API_PATH: &str = "/contacts";
const GROUPS_API_PATH: &str = "/groups";

// We'll create the full paths in the init function since we can't use format!() in constants

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

// Request enum for deserializing HTTP server messages
#[derive(Debug, Clone, Serialize, Deserialize, process_macros::SerdeJsonInto)]
struct GroupMessageRequest {
    group_id: String,
    message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, process_macros::SerdeJsonInto)]
struct AddContactRequest {
    id: String,
    name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, process_macros::SerdeJsonInto)]
struct GetMessagesRequest {}

#[derive(Debug, Clone, Serialize, Deserialize, process_macros::SerdeJsonInto)]
struct GetContactsRequest {}

#[derive(Debug, Clone, Serialize, Deserialize, process_macros::SerdeJsonInto)]
struct GetGroupsRequest {}

#[derive(Debug, Clone, Serialize, Deserialize, process_macros::SerdeJsonInto)]
struct GetGroupMessagesRequest {
    group_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, process_macros::SerdeJsonInto)]
#[serde(untagged)]
enum RequestType {
    ChatRequest(HyperwareChatRequest),
    HttpRequest(HttpServerRequest),
    GroupMessage { GroupMessage: GroupMessageRequest },
    CreateGroup { CreateGroup: CreateGroupRequest },
    AddContact { AddContact: AddContactRequest },
    GetMessages { GetMessages: GetMessagesRequest },
    GetContacts { GetContacts: GetContactsRequest },
    GetGroups { GetGroups: GetGroupsRequest },
    GetGroupMessages { GetGroupMessages: GetGroupMessagesRequest },
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
        // We'll iterate through all the messages we've stored
        // This is a simplified implementation that assumes we've actually interacted with contacts
        // In a real app, we would have a way to list all keys in the KV store
        
        // Use a temporary HashMap to store the result
        let mut messages = HashMap::new();
        
        // Get a list of all contacts we've messaged 
        // (Ideally we'd get this from the KV store, but for simplicity we'll just return what we have)
        let contacts = vec!["world".to_string(), "hello".to_string()]; // Example contacts
        
        // Get messages for each contact
        for contact in contacts {
            if let Ok(contact_messages) = self.get_messages(&contact) {
                if !contact_messages.is_empty() {
                    messages.insert(contact, contact_messages);
                }
            }
        }
        
        Ok(messages)
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
            
            // Log the raw request for debugging
            let request_str = String::from_utf8_lossy(&blob.bytes);
            info!("Received WebSocket message RAW: {}", request_str);

            let our_addr = standard::our();
            
            // Try to parse as RequestType to handle various message types
            match serde_json::from_slice::<RequestType>(&blob.bytes) {
                Ok(RequestType::HttpRequest(_)) => {
                    // This should not happen in a WebSocket message
                    info!("Received HttpRequest via WebSocket - this is unexpected");
                },
                Ok(RequestType::ChatRequest(request)) => {
                    handle_chat_request(
                        &our_addr,
                        &make_http_address(&our_addr),
                        request,
                        true,
                        store,
                        server,
                    )?;
                },
                Ok(RequestType::GroupMessage { GroupMessage: group_msg }) => {
                    info!("Received group message via WebSocket: {:?}", group_msg);
                    
                    // Check if group exists and user is a member
                    if let Ok(Some(group)) = store.get_group(&group_msg.group_id) {
                        if !group.members.contains(&our_addr.node().to_string()) {
                            info!("User is not a member of group {}", group_msg.group_id);
                            return Ok(());
                        }
                        
                        // Add message to group
                        let group_message = GroupMessage {
                            group_id: group_msg.group_id.clone(),
                            author: our_addr.node().to_string(),
                            content: group_msg.message.clone(),
                            timestamp: get_timestamp(),
                        };
                        
                        if let Ok(_) = store.add_group_message(&group_msg.group_id, group_message.clone()) {
                            // Send WebSocket message to update UI
                            let update_blob = LazyLoadBlob {
                                mime: Some("application/json".to_string()),
                                bytes: serde_json::to_vec(&serde_json::json!({
                                    "NewGroupMessage": {
                                        "group_id": group_msg.group_id,
                                        "author": our_addr.node().to_string(),
                                        "content": group_msg.message,
                                        "timestamp": group_message.timestamp,
                                    }
                                }))
                                .unwrap(),
                            };
                            server.ws_push_all_channels(WS_PATH, WsMessageType::Text, update_blob);
                        }
                    } else {
                        info!("Group not found: {}", group_msg.group_id);
                    }
                },
                Ok(RequestType::CreateGroup { CreateGroup: create_req }) => {
                    info!("Received create group request via WebSocket: {:?}", create_req);
                    
                    let members: HashSet<String> = create_req.members.into_iter().collect();
                    
                    // Add the creator to the group members
                    let mut all_members = members.clone();
                    all_members.insert(our_addr.node().to_string());
                    
                    match store.create_group(&create_req.name, all_members, our_addr.node()) {
                        Ok(group) => {
                            // Send WebSocket message to update UI
                            let update_blob = LazyLoadBlob {
                                mime: Some("application/json".to_string()),
                                bytes: serde_json::to_vec(&serde_json::json!({
                                    "NewGroup": group
                                }))
                                .unwrap(),
                            };
                            server.ws_push_all_channels(WS_PATH, WsMessageType::Text, update_blob);
                        }
                        Err(e) => {
                            info!("Failed to create group: {}", e);
                        }
                    }
                },
                Ok(RequestType::AddContact { AddContact: contact }) => {
                    info!("Received add contact request via WebSocket: {:?}", contact);
                    
                    // In a real implementation, we would add the contact to a contacts database
                    // For now, we'll just acknowledge it with a WebSocket message
                    let update_blob = LazyLoadBlob {
                        mime: Some("application/json".to_string()),
                        bytes: serde_json::to_vec(&serde_json::json!({
                            "ContactAdded": {
                                "id": contact.id,
                                "name": contact.name
                            }
                        }))
                        .unwrap(),
                    };
                    server.ws_push_all_channels(WS_PATH, WsMessageType::Text, update_blob);
                },
                Ok(RequestType::GetMessages { .. }) => {
                    info!("Received GetMessages request via WebSocket");
                    
                    // Get all messages and send them back
                    match store.get_all_messages() {
                        Ok(messages) => {
                            let update_blob = LazyLoadBlob {
                                mime: Some("application/json".to_string()),
                                bytes: serde_json::to_vec(&serde_json::json!({
                                    "Messages": messages
                                }))
                                .unwrap(),
                            };
                            server.ws_push_all_channels(WS_PATH, WsMessageType::Text, update_blob);
                        }
                        Err(e) => {
                            info!("Failed to get messages: {}", e);
                        }
                    }
                },
                Ok(RequestType::GetContacts { .. }) => {
                    info!("Received GetContacts request via WebSocket");
                    
                    // In a real implementation, we would get contacts from a database
                    // For now, just send an empty list
                    let update_blob = LazyLoadBlob {
                        mime: Some("application/json".to_string()),
                        bytes: serde_json::to_vec(&serde_json::json!({
                            "Contacts": []
                        }))
                        .unwrap(),
                    };
                    server.ws_push_all_channels(WS_PATH, WsMessageType::Text, update_blob);
                },
                Ok(RequestType::GetGroups { .. }) => {
                    info!("Received GetGroups request via WebSocket");
                    
                    // Get all groups
                    match store.get_all_groups() {
                        Ok(groups) => {
                            let update_blob = LazyLoadBlob {
                                mime: Some("application/json".to_string()),
                                bytes: serde_json::to_vec(&serde_json::json!({
                                    "Groups": groups
                                }))
                                .unwrap(),
                            };
                            server.ws_push_all_channels(WS_PATH, WsMessageType::Text, update_blob);
                            
                            // For each group, also send its messages
                            for group in groups {
                                if let Ok(messages) = store.get_group_messages(&group.id) {
                                    let group_msg_blob = LazyLoadBlob {
                                        mime: Some("application/json".to_string()),
                                        bytes: serde_json::to_vec(&serde_json::json!({
                                            "GroupMessages": {
                                                "group_id": group.id,
                                                "messages": messages
                                            }
                                        }))
                                        .unwrap(),
                                    };
                                    server.ws_push_all_channels(WS_PATH, WsMessageType::Text, group_msg_blob);
                                }
                            }
                        }
                        Err(e) => {
                            info!("Failed to get groups: {}", e);
                        }
                    }
                },
                Ok(RequestType::GetGroupMessages { GetGroupMessages: req }) => {
                    info!("Received GetGroupMessages request via WebSocket for group: {}", req.group_id);
                    
                    // Get messages for a specific group
                    match store.get_group_messages(&req.group_id) {
                        Ok(messages) => {
                            let update_blob = LazyLoadBlob {
                                mime: Some("application/json".to_string()),
                                bytes: serde_json::to_vec(&serde_json::json!({
                                    "GroupMessages": {
                                        "group_id": req.group_id,
                                        "messages": messages
                                    }
                                }))
                                .unwrap(),
                            };
                            server.ws_push_all_channels(WS_PATH, WsMessageType::Text, update_blob);
                        }
                        Err(e) => {
                            info!("Failed to get group messages: {}", e);
                        }
                    }
                },
                Err(e) => {
                    info!("Failed to parse WebSocket message: {}", e);
                    
                    // Try to parse as a raw JSON object for manual handling
                    if let Ok(json_value) = serde_json::from_slice::<serde_json::Value>(&blob.bytes) {
                        info!("Parsed as raw JSON: {:?}", json_value);
                        
                        // This could be the payload from api.send({ data: ... })
                        if let Some(data_str) = json_value.get("data").and_then(|v| v.as_str()) {
                            info!("Found data field, trying to parse as JSON: {}", data_str);
                            if let Ok(inner_json) = serde_json::from_str::<serde_json::Value>(data_str) {
                                info!("Parsed inner JSON: {:?}", inner_json);
                                
                                // Check if inner JSON contains a Send field
                                if let Some(send) = inner_json.get("Send") {
                                    if let (Some(target), Some(message)) = (
                                        send.get("target").and_then(|v| v.as_str()), 
                                        send.get("message").and_then(|v| v.as_str())
                                    ) {
                                        info!("Found Send message: target={}, message={}", target, message);
                                        
                                        let request = HyperwareChatRequest::Send(SendRequest {
                                            target: target.to_string(),
                                            message: message.to_string(),
                                        });
                                        
                                        handle_chat_request(
                                            &our_addr,
                                            &make_http_address(&our_addr),
                                            request,
                                            true,
                                            store,
                                            server,
                                        )?;
                                        return Ok(());
                                    }
                                }
                            }
                        }
                        
                        // Try direct parsing of the original message
                        if let Some(send) = json_value.get("Send") {
                            if let (Some(target), Some(message)) = (
                                send.get("target").and_then(|v| v.as_str()), 
                                send.get("message").and_then(|v| v.as_str())
                            ) {
                                info!("Found Send message: target={}, message={}", target, message);
                                
                                let request = HyperwareChatRequest::Send(SendRequest {
                                    target: target.to_string(),
                                    message: message.to_string(),
                                });
                                
                                handle_chat_request(
                                    &our_addr,
                                    &make_http_address(&our_addr),
                                    request,
                                    true,
                                    store,
                                    server,
                                )?;
                                return Ok(());
                            }
                        }
                    }
                }
            }
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
                        
                        // Log the raw request to help with debugging
                        let request_str = String::from_utf8_lossy(&blob.bytes);
                        info!("Received HTTP POST request: {}", request_str);
                        
                        let our_addr = standard::our();
                        
                        // Try to parse as HyperwareChatRequest directly first
                        if let Ok(request) = serde_json::from_slice::<HyperwareChatRequest>(&blob.bytes) {
                            handle_chat_request(
                                &our_addr,
                                &make_http_address(&our_addr),
                                request,
                                true,
                                store,
                                server,
                            )?;
                        } 
                        // Try to parse as a JSON object that might contain a Send field
                        else if let Ok(json_value) = serde_json::from_slice::<serde_json::Value>(&blob.bytes) {
                            if let Some(send_obj) = json_value.get("Send") {
                                if let (Some(target), Some(message)) = (
                                    send_obj.get("target").and_then(|v| v.as_str()), 
                                    send_obj.get("message").and_then(|v| v.as_str())
                                ) {
                                    let request = HyperwareChatRequest::Send(SendRequest {
                                        target: target.to_string(),
                                        message: message.to_string(),
                                    });
                                    
                                    handle_chat_request(
                                        &our_addr,
                                        &make_http_address(&our_addr),
                                        request,
                                        true,
                                        store,
                                        server,
                                    )?;
                                } else {
                                    info!("JSON has Send field but missing target or message");
                                    send_response(StatusCode::BAD_REQUEST, None, vec![]);
                                    return Ok(());
                                }
                            } else {
                                info!("JSON doesn't contain Send field: {:?}", json_value);
                                send_response(StatusCode::BAD_REQUEST, None, vec![]);
                                return Ok(());
                            }
                        } else {
                            info!("Failed to parse HTTP blob as either HyperwareChatRequest or JSON");
                            send_response(StatusCode::BAD_REQUEST, None, vec![]);
                            return Ok(());
                        }

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
                            let our_addr = standard::our();
                            let mut all_members = members.clone();
                            all_members.insert(our_addr.node().to_string());
                            
                            match store.create_group(&create_group_req.name, all_members, our_addr.node()) {
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
                                        let our_addr = standard::our();
                                        if !group.members.contains(&our_addr.node().to_string()) {
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
                                            author: our_addr.node().to_string(),
                                            content: msg_req.message.clone(),
                                            timestamp: get_timestamp(),
                                        };
                                        
                                        match store.add_group_message(&msg_req.group_id, group_message.clone()) {
                                            Ok(_) => {
                                                // Notify all group members
                                                for member in &group.members {
                                                    if member != &our_addr.node().to_string() {
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
                                                            "author": our_addr.node().to_string(),
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
    request: HyperwareChatRequest,
    is_http: bool,
    store: &ChatStore,
    server: &HttpServer,
) -> anyhow::Result<()> {
    match request {
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
                let request = HyperwareChatRequest::Send(SendRequest {
                    target: target.clone(),
                    message: message.clone(),
                });
                Request::new()
                    .target((target, "hyperware-chat", "hyperware-chat", "template.os"))
                    .body(request)
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
    message: &ProcessMessage,
    store: &ChatStore,
    server: &mut HttpServer,
) -> anyhow::Result<()> {
    if !message.is_request() {
        return Ok(());
    }

    let body = message.body();
    let source = message.source();
    let our_addr = standard::our();

    // Try to parse the message as either a chat request or HTTP request
    match serde_json::from_slice::<RequestType>(body) {
        Ok(RequestType::HttpRequest(request)) => {
            handle_http_server_request(body, request, store, server)?;
        },
        Ok(RequestType::ChatRequest(chat_request)) => {
            handle_chat_request(&our_addr, source, chat_request, false, store, server)?;
        },
        // Handle other RequestType variants
        Ok(RequestType::GroupMessage { GroupMessage: group_msg }) => {
            info!("Received GroupMessage in handle_message, this is unexpected: {:?}", group_msg);
        },
        Ok(RequestType::CreateGroup { CreateGroup: create_req }) => {
            info!("Received CreateGroup in handle_message, this is unexpected: {:?}", create_req);
        },
        Ok(RequestType::AddContact { AddContact: contact }) => {
            info!("Received AddContact in handle_message, this is unexpected: {:?}", contact);
        },
        Ok(RequestType::GetMessages { .. }) => {
            info!("Received GetMessages in handle_message, this is unexpected");
        },
        Ok(RequestType::GetContacts { .. }) => {
            info!("Received GetContacts in handle_message, this is unexpected");
        },
        Ok(RequestType::GetGroups { .. }) => {
            info!("Received GetGroups in handle_message, this is unexpected");
        },
        Ok(RequestType::GetGroupMessages { .. }) => {
            info!("Received GetGroupMessages in handle_message, this is unexpected");
        },
        Err(e) => {
            // If from HTTP server, try to parse directly as HttpServerRequest
            if source == &make_http_address(&our_addr) {
                match serde_json::from_slice::<HttpServerRequest>(body) {
                    Ok(request) => {
                        handle_http_server_request(body, request, store, server)?;
                    },
                    Err(_) => {
                        error!("couldn't parse message from http_server: {body:?}, error: {e}");
                    }
                }
            } else {
                // Try to parse as HyperwareChatRequest
                match serde_json::from_slice::<HyperwareChatRequest>(body) {
                    Ok(chat_request) => {
                        handle_chat_request(&our_addr, source, chat_request, false, store, server)?;
                    },
                    Err(_) => {
                        error!("couldn't parse message: {body:?}, error: {e}");
                    }
                }
            }
        }
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

    // Create full paths with process prefix
    let full_ws_path = format!("/{}", PROCESS_PATH);
    let full_http_api_path = format!("/{}{}", PROCESS_PATH, HTTP_API_PATH);
    let full_contacts_api_path = format!("/{}{}", PROCESS_PATH, CONTACTS_API_PATH);
    let full_groups_api_path = format!("/{}{}", PROCESS_PATH, GROUPS_API_PATH);
    
    info!("Binding paths: {}, {}, {}, {}", full_ws_path, full_http_api_path, full_contacts_api_path, full_groups_api_path);
    
    // Bind UI files to routes with index.html at "/"
    server
        .serve_ui("ui", vec!["/", &full_ws_path], HttpBindingConfig::default())
        .expect("failed to serve UI");
    
    // API endpoints with full process paths
    server
        .bind_http_path(&full_http_api_path, HttpBindingConfig::default())
        .expect("failed to bind messages API");
    server
        .bind_http_path(&full_contacts_api_path, HttpBindingConfig::default())
        .expect("failed to bind contacts API");
    server
        .bind_http_path(&full_groups_api_path, HttpBindingConfig::default())
        .expect("failed to bind groups API");
    
    // Also bind the short paths for backward compatibility
    server
        .bind_http_path(HTTP_API_PATH, HttpBindingConfig::default())
        .expect("failed to bind short messages API");
    server
        .bind_http_path(CONTACTS_API_PATH, HttpBindingConfig::default())
        .expect("failed to bind short contacts API");
    server
        .bind_http_path(GROUPS_API_PATH, HttpBindingConfig::default())
        .expect("failed to bind short groups API");
    
    // WebSocket for real-time updates
    server
        .bind_ws_path(&full_ws_path, WsBindingConfig::default())
        .expect("failed to bind WS API");
    server
        .bind_ws_path(WS_PATH, WsBindingConfig::default())
        .expect("failed to bind short WS API");

    loop {
        match await_message() {
            Err(send_error) => error!("got SendError: {send_error}"),
            Ok(ref message) => {
                match handle_message(message, &store, &mut server) {
                    Ok(_) => {}
                    Err(e) => error!("got error while handling message: {e:?}"),
                }
            }
        }
    }
}
