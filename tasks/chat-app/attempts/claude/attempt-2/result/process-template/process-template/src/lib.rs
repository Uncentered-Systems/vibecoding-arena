use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use hyperware_process_lib::logging::{error, info, init_logging, Level};
use hyperware_process_lib::{
    await_message, call_init,
    http::server::{
        HttpBindingConfig, HttpServer,
        WsBindingConfig,
    },
    Address, Message
};
use shared_types::{MessageChannel, MessageType, MessageLog, AppConfig, AppState};

mod message_handlers;
use message_handlers::*;

wit_bindgen::generate!({
    path: "target/wit",
    world: "process-template-template-dot-os-v0",
    generate_unused_types: true,
    additional_derives: [serde::Deserialize, serde::Serialize, process_macros::SerdeJsonInto],
});

const WS_PATH: &str = "/ws";

fn bind_http_endpoints(server: &mut HttpServer) {
    let public_config = HttpBindingConfig::new(false, false, false, None);
    
    // Define main API endpoint - will handle all requests
    server.bind_http_path("/api", public_config.clone())
        .expect("failed to bind main HTTP API path");
}

fn get_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// Helper function to log a message and update counts
fn log_message(
    state: &mut AppState,
    source: String,
    channel: MessageChannel,
    message_type: MessageType,
    content: Option<String>,
) {
    // Add to message history
    state.message_history.push(MessageLog {
        source,
        channel: channel.clone(),
        message_type,
        content: if state.config.log_content { content } else { None },
        timestamp: get_timestamp(),
    });
    
    // Update message count for this channel
    *state.message_counts.entry(channel).or_insert(0) += 1;
    
    // Trim history if needed
    if state.message_history.len() > state.config.max_history {
        state.message_history.remove(0);
    }
}

fn handle_message(
    message: &Message,
    our: &Address,
    state: &mut AppState,
    server: &mut HttpServer,
) -> anyhow::Result<()> {
    match message.source() {
        // Handling HTTP requests
        source if source == &make_http_address(our) => {
            handle_http_server_request(our, message.body(), state, server)
        }
        // Handling WebSocket messages
        source if source == &make_ws_address(our) => {
            handle_ws_server_request(our, message.body(), state, server)
        }
        // Handling timer messages
        source if source == &make_timer_address(our) => {
            handle_timer_message(message.body(), state, server)
        }
        // Handling terminal messages
        source if source == &make_terminal_address(our) => {
            handle_terminal_message(message.body(), state, server)
        }
        // Handling internal messages
        source if source.node == our.node => {
            handle_internal_message(source, message.body(), state, server)
        }
        // Handling external messages (from other nodes)
        _ => {
            handle_external_message(message.source(), message.body(), state, server)
        },
    }
}

call_init!(init);
fn init(our: Address) {
    // Initialize logging
    init_logging(Level::DEBUG, Level::INFO, None, None, None).unwrap();
    info!("Hyperchat application starting");

    // Initialize application state
    let mut state = AppState {
        config: AppConfig {
            max_history: 100,
            log_content: true,
        },
        message_counts: HashMap::new(),
        ..Default::default()
    };

    // Set up HTTP server
    let mut server = HttpServer::new(5);
    let http_config = HttpBindingConfig::default();
    bind_http_endpoints(&mut server);

    // Bind UI files
    server
        .serve_ui("ui", vec!["/"], http_config.clone())
        .expect("failed to serve UI");

    // Bind WebSocket for real-time communication
    server
        .bind_ws_path(WS_PATH, WsBindingConfig::default())
        .expect("failed to bind WebSocket API");

    // Log initialization
    log_message(
        &mut state,
        "System".to_string(),
        MessageChannel::Internal,
        MessageType::Other("Initialization".to_string()),
        Some("Hyperchat application started".to_string()),
    );

    // Main message loop
    loop {
        match await_message() {
            Err(send_error) => {
                error!("got SendError: {send_error}");
                log_message(
                    &mut state,
                    "System".to_string(),
                    MessageChannel::Internal,
                    MessageType::Other("Error".to_string()),
                    Some(format!("SendError: {}", send_error)),
                );
            },
            Ok(ref message) => {
                match handle_message(message, &our, &mut state, &mut server) {
                    Ok(_) => {}
                    Err(e) => {
                        error!("got error while handling message: {e:?}");
                        log_message(
                            &mut state,
                            "System".to_string(),
                            MessageChannel::Internal,
                            MessageType::Other("Error".to_string()),
                            Some(format!("Error handling message: {}", e)),
                        );
                    }
                }
            }
        }
    }
}