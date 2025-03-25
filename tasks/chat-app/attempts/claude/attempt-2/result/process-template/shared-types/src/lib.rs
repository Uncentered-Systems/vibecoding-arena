use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Message source/channel type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MessageChannel {
    /// WebSocket messages
    WebSocket,
    /// HTTP API requests
    HttpApi,
    /// Internal process messages
    Internal,
    /// External node messages
    External,
    /// Timer events
    Timer,
    /// Terminal commands
    Terminal,
}

/// Message type for categorization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    /// WebSocket connection opened
    WebSocketOpen,
    /// WebSocket connection closed
    WebSocketClose,
    /// WebSocket message received
    WebSocketPushA,
    /// Another type of WebSocket message
    WebSocketPushB,
    /// HTTP GET request
    HttpGet,
    /// HTTP POST request
    HttpPost,
    /// Timer tick event
    TimerTick,
    /// Local process request
    LocalRequest,
    /// Remote node request
    RemoteRequest,
    /// Response to our request
    ResponseReceived,
    /// Terminal command
    TerminalCommand,
    /// Other message type
    Other(String),
}

/// Log entry for tracking messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageLog {
    /// Source of the message
    pub source: String,
    /// Channel the message came through
    pub channel: MessageChannel,
    /// Type of the message
    pub message_type: MessageType,
    /// Message content (if available)
    pub content: Option<String>,
    /// Timestamp when the message was received
    pub timestamp: u64,
}

/// Represents the application state
#[derive(Debug, Clone, Default)]
pub struct AppState {
    /// Tracks message history for all channels
    pub message_history: Vec<MessageLog>,
    /// Message counts by channel
    pub message_counts: HashMap<MessageChannel, usize>,
    /// Configuration settings
    pub config: AppConfig,
    /// Connected WebSocket clients (channel_id -> path)
    pub connected_clients: HashMap<u32, String>,
    /// Connected users' node addresses (channel_id -> node_address)
    pub connected_users: HashMap<u32, String>,
}

/// Configuration for the application
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Maximum number of messages to keep in history
    pub max_history: usize,
    /// Whether to log message content
    pub log_content: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            max_history: 100,
            log_content: true,
        }
    }
}

/// API request types for Hyperchat application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiRequest {
    /// Get application status
    GetStatus,
    /// Get message history
    GetHistory,
    /// Send a custom message
    CustomMessage {
        /// Type of message
        message_type: String,
        /// Message content
        content: String,
    },
    /// Create a new contact
    CreateContact {
        /// Contact name
        name: String,
        /// Contact address
        address: String,
    },
    /// List all contacts
    ListContacts,
    /// Start a new conversation
    StartConversation {
        /// Conversation name
        name: String,
        /// Conversation participants (addresses)
        participants: Vec<String>,
    },
    /// Send a message to a conversation
    SendMessage {
        /// Conversation ID
        conversation_id: String,
        /// Message content
        content: String,
    },
}

/// API response types for Hyperchat application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiResponse {
    /// Success response with optional data
    Success {
        /// Response data (if any)
        data: Option<serde_json::Value>,
    },
    /// Error response with message
    Error {
        /// Error message
        message: String,
    },
}

/// WebSocket message for real-time updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    /// Type of message
    pub message_type: String,
    /// Message data
    pub data: serde_json::Value,
}