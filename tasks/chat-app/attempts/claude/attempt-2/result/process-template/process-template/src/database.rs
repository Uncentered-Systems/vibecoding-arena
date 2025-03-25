use anyhow::{anyhow, Result};
use hyperware_process_lib::logging::{debug, error, info};
use hyperware_process_lib::sqlite::{RowId, SqliteQuery, SqliteResult, Transaction};
use shared_types::{Contact, Conversation, ConversationMember, ChatMessage};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

const DB_NAME: &str = "hyperchat.db";

/// Gets the current timestamp in seconds since the Unix epoch
pub fn get_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Generate a unique ID for a conversation
pub fn generate_id() -> String {
    Uuid::new_v4().to_string()
}

/// Initialize the database by creating the necessary tables
pub fn initialize_database() -> Result<()> {
    info!("Initializing database");
    
    // Create contacts table
    let create_contacts_table = r#"
        CREATE TABLE IF NOT EXISTS contacts (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            node_address TEXT UNIQUE NOT NULL,
            avatar_path TEXT,
            status TEXT DEFAULT 'offline',
            last_seen INTEGER,
            created_at INTEGER NOT NULL
        )
    "#;
    
    // Create conversations table
    let create_conversations_table = r#"
        CREATE TABLE IF NOT EXISTS conversations (
            id TEXT PRIMARY KEY,
            type TEXT NOT NULL,
            title TEXT,
            created_at INTEGER NOT NULL,
            last_message_at INTEGER
        )
    "#;
    
    // Create conversation members table
    let create_members_table = r#"
        CREATE TABLE IF NOT EXISTS conversation_members (
            conversation_id TEXT NOT NULL,
            member_address TEXT NOT NULL,
            join_timestamp INTEGER NOT NULL,
            is_admin INTEGER DEFAULT 0,
            PRIMARY KEY (conversation_id, member_address)
        )
    "#;
    
    // Create messages table
    let create_messages_table = r#"
        CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY,
            conversation_id TEXT NOT NULL,
            sender_id TEXT NOT NULL,
            content TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            read_status INTEGER DEFAULT 0,
            has_attachment INTEGER DEFAULT 0,
            attachment_path TEXT
        )
    "#;
    
    let transaction = Transaction::new()?;
    
    // Execute all table creation statements
    transaction.execute(DB_NAME, SqliteQuery::new(create_contacts_table, vec![]))?;
    transaction.execute(DB_NAME, SqliteQuery::new(create_conversations_table, vec![]))?;
    transaction.execute(DB_NAME, SqliteQuery::new(create_members_table, vec![]))?;
    transaction.execute(DB_NAME, SqliteQuery::new(create_messages_table, vec![]))?;
    
    transaction.commit()?;
    
    info!("Database initialization complete");
    Ok(())
}

/// Contact management functions
pub struct ContactRepository;

impl ContactRepository {
    /// Add a new contact
    pub fn add_contact(name: &str, node_address: &str) -> Result<i64> {
        let timestamp = get_timestamp();
        let query = SqliteQuery::new(
            "INSERT INTO contacts (name, node_address, status, created_at) VALUES (?, ?, 'offline', ?)",
            vec![name.into(), node_address.into(), timestamp.into()],
        );
        
        let transaction = Transaction::new()?;
        let result = transaction.execute(DB_NAME, query)?;
        let commit_result = transaction.commit()?;
        
        match result {
            SqliteResult::Write(RowId(id)) => Ok(id),
            _ => Err(anyhow!("Failed to add contact")),
        }
    }
    
    /// Get all contacts
    pub fn get_contacts() -> Result<Vec<Contact>> {
        let query = SqliteQuery::new(
            "SELECT id, name, node_address, avatar_path, status, last_seen, created_at FROM contacts ORDER BY name",
            vec![],
        );
        
        let transaction = Transaction::new()?;
        let result = transaction.read(DB_NAME, query)?;
        let commit_result = transaction.commit()?;
        
        match result {
            SqliteResult::Read(rows) => {
                let contacts = rows.iter().map(|row| {
                    Contact {
                        id: row.get("id").map(|v| v.as_i64().unwrap_or(0)),
                        name: row.get("name").map(|v| v.as_string().unwrap_or_default()).unwrap_or_default(),
                        node_address: row.get("node_address").map(|v| v.as_string().unwrap_or_default()).unwrap_or_default(),
                        avatar_path: row.get("avatar_path").and_then(|v| v.as_string()),
                        status: row.get("status").map(|v| v.as_string().unwrap_or("offline".to_string())).unwrap_or_else(|| "offline".to_string()),
                        last_seen: row.get("last_seen").and_then(|v| v.as_i64().map(|ts| ts as u64)),
                        created_at: row.get("created_at").map(|v| v.as_i64().unwrap_or(0) as u64).unwrap_or_else(|| get_timestamp()),
                    }
                }).collect();
                
                Ok(contacts)
            },
            _ => Err(anyhow!("Failed to fetch contacts")),
        }
    }
    
    /// Get contacts by status
    pub fn get_contacts_by_status(status: &str) -> Result<Vec<Contact>> {
        let query = SqliteQuery::new(
            "SELECT id, name, node_address, avatar_path, status, last_seen, created_at FROM contacts WHERE status = ? ORDER BY name",
            vec![status.into()],
        );
        
        let transaction = Transaction::new()?;
        let result = transaction.read(DB_NAME, query)?;
        let commit_result = transaction.commit()?;
        
        match result {
            SqliteResult::Read(rows) => {
                let contacts = rows.iter().map(|row| {
                    Contact {
                        id: row.get("id").map(|v| v.as_i64().unwrap_or(0)),
                        name: row.get("name").map(|v| v.as_string().unwrap_or_default()).unwrap_or_default(),
                        node_address: row.get("node_address").map(|v| v.as_string().unwrap_or_default()).unwrap_or_default(),
                        avatar_path: row.get("avatar_path").and_then(|v| v.as_string()),
                        status: row.get("status").map(|v| v.as_string().unwrap_or("offline".to_string())).unwrap_or_else(|| "offline".to_string()),
                        last_seen: row.get("last_seen").and_then(|v| v.as_i64().map(|ts| ts as u64)),
                        created_at: row.get("created_at").map(|v| v.as_i64().unwrap_or(0) as u64).unwrap_or_else(|| get_timestamp()),
                    }
                }).collect();
                
                Ok(contacts)
            },
            _ => Err(anyhow!("Failed to fetch contacts by status")),
        }
    }
    
    /// Get a contact by node address
    pub fn get_contact_by_node_address(node_address: &str) -> Result<Contact> {
        let query = SqliteQuery::new(
            "SELECT id, name, node_address, avatar_path, status, last_seen, created_at FROM contacts WHERE node_address = ?",
            vec![node_address.into()],
        );
        
        let transaction = Transaction::new()?;
        let result = transaction.read(DB_NAME, query)?;
        let commit_result = transaction.commit()?;
        
        match result {
            SqliteResult::Read(rows) => {
                if rows.is_empty() {
                    return Err(anyhow!("Contact not found"));
                }
                
                let row = &rows[0];
                let contact = Contact {
                    id: row.get("id").map(|v| v.as_i64().unwrap_or(0)),
                    name: row.get("name").map(|v| v.as_string().unwrap_or_default()).unwrap_or_default(),
                    node_address: row.get("node_address").map(|v| v.as_string().unwrap_or_default()).unwrap_or_default(),
                    avatar_path: row.get("avatar_path").and_then(|v| v.as_string()),
                    status: row.get("status").map(|v| v.as_string().unwrap_or("offline".to_string())).unwrap_or_else(|| "offline".to_string()),
                    last_seen: row.get("last_seen").and_then(|v| v.as_i64().map(|ts| ts as u64)),
                    created_at: row.get("created_at").map(|v| v.as_i64().unwrap_or(0) as u64).unwrap_or_else(|| get_timestamp()),
                };
                
                Ok(contact)
            },
            _ => Err(anyhow!("Failed to fetch contact")),
        }
    }
    
    /// Update a contact's status
    pub fn update_contact_status(node_address: &str, status: &str) -> Result<()> {
        let timestamp = get_timestamp();
        let query = SqliteQuery::new(
            "UPDATE contacts SET status = ?, last_seen = ? WHERE node_address = ?",
            vec![status.into(), timestamp.into(), node_address.into()],
        );
        
        let transaction = Transaction::new()?;
        let result = transaction.execute(DB_NAME, query)?;
        let commit_result = transaction.commit()?;
        
        match result {
            SqliteResult::Write(_) => Ok(()),
            _ => Err(anyhow!("Failed to update contact status")),
        }
    }
    
    /// Delete a contact
    pub fn delete_contact(id: i64) -> Result<()> {
        let query = SqliteQuery::new(
            "DELETE FROM contacts WHERE id = ?",
            vec![id.into()],
        );
        
        let transaction = Transaction::new()?;
        let result = transaction.execute(DB_NAME, query)?;
        let commit_result = transaction.commit()?;
        
        match result {
            SqliteResult::Write(_) => Ok(()),
            _ => Err(anyhow!("Failed to delete contact")),
        }
    }
}

/// Conversation management functions
pub struct ConversationRepository;

impl ConversationRepository {
    /// Create a new conversation
    pub fn create_conversation(
        type_: &str,
        title: Option<&str>,
        member_addresses: &[String],
    ) -> Result<String> {
        let conversation_id = generate_id();
        let timestamp = get_timestamp();
        let transaction = Transaction::new()?;
        
        // Insert conversation
        let conversation_query = SqliteQuery::new(
            "INSERT INTO conversations (id, type, title, created_at) VALUES (?, ?, ?, ?)",
            vec![
                conversation_id.clone().into(),
                type_.into(),
                title.map(|t| t.into()).unwrap_or_else(|| "NULL".into()),
                timestamp.into(),
            ],
        );
        
        transaction.execute(DB_NAME, conversation_query)?;
        
        // Add members
        for address in member_addresses {
            let member_query = SqliteQuery::new(
                "INSERT INTO conversation_members (conversation_id, member_address, join_timestamp, is_admin) VALUES (?, ?, ?, ?)",
                vec![
                    conversation_id.clone().into(),
                    address.clone().into(),
                    timestamp.into(),
                    // First member is admin in group chats
                    if type_ == "group" && address == &member_addresses[0] { 1i64.into() } else { 0i64.into() },
                ],
            );
            
            transaction.execute(DB_NAME, member_query)?;
        }
        
        transaction.commit()?;
        
        Ok(conversation_id)
    }
    
    /// Get all conversations for a user
    pub fn get_conversations_for_user(user_address: &str) -> Result<Vec<Conversation>> {
        let query = SqliteQuery::new(
            r#"
            SELECT 
                c.id, c.type, c.title, c.created_at, c.last_message_at,
                (SELECT content FROM messages WHERE conversation_id = c.id ORDER BY timestamp DESC LIMIT 1) as last_message,
                (SELECT COUNT(*) FROM messages WHERE conversation_id = c.id AND read_status = 0) as unread_count
            FROM conversations c
            JOIN conversation_members cm ON c.id = cm.conversation_id
            WHERE cm.member_address = ?
            ORDER BY c.last_message_at DESC NULLS LAST, c.created_at DESC
            "#,
            vec![user_address.into()],
        );
        
        let transaction = Transaction::new()?;
        let result = transaction.read(DB_NAME, query)?;
        let commit_result = transaction.commit()?;
        
        match result {
            SqliteResult::Read(rows) => {
                let conversations = rows.iter().map(|row| {
                    Conversation {
                        id: row.get("id").map(|v| v.as_string().unwrap_or_default()).unwrap_or_default(),
                        type_: row.get("type").map(|v| v.as_string().unwrap_or_default()).unwrap_or_default(),
                        title: row.get("title").and_then(|v| v.as_string()),
                        created_at: row.get("created_at").map(|v| v.as_i64().unwrap_or(0) as u64).unwrap_or_else(|| get_timestamp()),
                        last_message_at: row.get("last_message_at").and_then(|v| v.as_i64().map(|ts| ts as u64)),
                        last_message: row.get("last_message").and_then(|v| v.as_string()),
                        unread_count: row.get("unread_count").map(|v| v.as_i64().unwrap_or(0)).unwrap_or(0),
                    }
                }).collect();
                
                Ok(conversations)
            },
            _ => Err(anyhow!("Failed to fetch conversations")),
        }
    }
    
    /// Get conversation members
    pub fn get_conversation_members(conversation_id: &str) -> Result<Vec<ConversationMember>> {
        let query = SqliteQuery::new(
            r#"
            SELECT 
                cm.conversation_id, cm.member_address, cm.join_timestamp, cm.is_admin,
                c.name
            FROM conversation_members cm
            LEFT JOIN contacts c ON cm.member_address = c.node_address
            WHERE cm.conversation_id = ?
            ORDER BY cm.is_admin DESC, c.name
            "#,
            vec![conversation_id.into()],
        );
        
        let transaction = Transaction::new()?;
        let result = transaction.read(DB_NAME, query)?;
        let commit_result = transaction.commit()?;
        
        match result {
            SqliteResult::Read(rows) => {
                let members = rows.iter().map(|row| {
                    ConversationMember {
                        conversation_id: row.get("conversation_id").map(|v| v.as_string().unwrap_or_default()).unwrap_or_default(),
                        member_address: row.get("member_address").map(|v| v.as_string().unwrap_or_default()).unwrap_or_default(),
                        join_timestamp: row.get("join_timestamp").map(|v| v.as_i64().unwrap_or(0) as u64).unwrap_or_else(|| get_timestamp()),
                        is_admin: row.get("is_admin").map(|v| v.as_i64().unwrap_or(0) == 1).unwrap_or(false),
                        name: row.get("name").and_then(|v| v.as_string()),
                    }
                }).collect();
                
                Ok(members)
            },
            _ => Err(anyhow!("Failed to fetch conversation members")),
        }
    }
    
    /// Add a member to a conversation
    pub fn add_conversation_member(conversation_id: &str, node_address: &str) -> Result<()> {
        let timestamp = get_timestamp();
        let query = SqliteQuery::new(
            "INSERT OR IGNORE INTO conversation_members (conversation_id, member_address, join_timestamp, is_admin) VALUES (?, ?, ?, 0)",
            vec![conversation_id.into(), node_address.into(), timestamp.into()],
        );
        
        let transaction = Transaction::new()?;
        let result = transaction.execute(DB_NAME, query)?;
        let commit_result = transaction.commit()?;
        
        match result {
            SqliteResult::Write(_) => Ok(()),
            _ => Err(anyhow!("Failed to add conversation member")),
        }
    }
}

/// Message management functions
pub struct MessageRepository;

impl MessageRepository {
    /// Add a new message
    pub fn add_message(
        conversation_id: &str,
        sender_id: &str,
        content: &str,
        has_attachment: bool,
        attachment_path: Option<&str>,
    ) -> Result<i64> {
        let timestamp = get_timestamp();
        let transaction = Transaction::new()?;
        
        // Insert message
        let message_query = SqliteQuery::new(
            "INSERT INTO messages (conversation_id, sender_id, content, timestamp, has_attachment, attachment_path) VALUES (?, ?, ?, ?, ?, ?)",
            vec![
                conversation_id.into(),
                sender_id.into(),
                content.into(),
                timestamp.into(),
                (if has_attachment { 1 } else { 0 }).into(),
                attachment_path.map(|p| p.into()).unwrap_or_else(|| "NULL".into()),
            ],
        );
        
        let message_result = transaction.execute(DB_NAME, message_query)?;
        
        // Update conversation last message time
        let update_query = SqliteQuery::new(
            "UPDATE conversations SET last_message_at = ? WHERE id = ?",
            vec![timestamp.into(), conversation_id.into()],
        );
        
        transaction.execute(DB_NAME, update_query)?;
        transaction.commit()?;
        
        match message_result {
            SqliteResult::Write(RowId(id)) => Ok(id),
            _ => Err(anyhow!("Failed to add message")),
        }
    }
    
    /// Get messages for a conversation
    pub fn get_messages(conversation_id: &str) -> Result<Vec<ChatMessage>> {
        let query = SqliteQuery::new(
            "SELECT id, conversation_id, sender_id, content, timestamp, read_status, has_attachment, attachment_path FROM messages WHERE conversation_id = ? ORDER BY timestamp ASC",
            vec![conversation_id.into()],
        );
        
        let transaction = Transaction::new()?;
        let result = transaction.read(DB_NAME, query)?;
        let commit_result = transaction.commit()?;
        
        match result {
            SqliteResult::Read(rows) => {
                let messages = rows.iter().map(|row| {
                    ChatMessage {
                        id: row.get("id").map(|v| v.as_i64().unwrap_or(0)),
                        conversation_id: row.get("conversation_id").map(|v| v.as_string().unwrap_or_default()).unwrap_or_default(),
                        sender_id: row.get("sender_id").map(|v| v.as_string().unwrap_or_default()).unwrap_or_default(),
                        content: row.get("content").map(|v| v.as_string().unwrap_or_default()).unwrap_or_default(),
                        timestamp: row.get("timestamp").map(|v| v.as_i64().unwrap_or(0) as u64).unwrap_or_else(|| get_timestamp()),
                        read_status: row.get("read_status").map(|v| v.as_i64().unwrap_or(0)).unwrap_or(0),
                        has_attachment: row.get("has_attachment").map(|v| v.as_i64().unwrap_or(0) == 1).unwrap_or(false),
                        attachment_path: row.get("attachment_path").and_then(|v| v.as_string()),
                    }
                }).collect();
                
                Ok(messages)
            },
            _ => Err(anyhow!("Failed to fetch messages")),
        }
    }
    
    /// Mark a message as read
    pub fn mark_message_read(message_id: i64) -> Result<()> {
        let query = SqliteQuery::new(
            "UPDATE messages SET read_status = 1 WHERE id = ?",
            vec![message_id.into()],
        );
        
        let transaction = Transaction::new()?;
        let result = transaction.execute(DB_NAME, query)?;
        let commit_result = transaction.commit()?;
        
        match result {
            SqliteResult::Write(_) => Ok(()),
            _ => Err(anyhow!("Failed to mark message as read")),
        }
    }
}