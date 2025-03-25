# Hyperchat: Hyperware Messaging Application Design

## Overview
Hyperchat is a messaging application built on the Hyperware platform, enabling users to communicate through direct messages and group chats. The application leverages node addresses as user identifiers and provides a responsive UI for chat interactions.

## Data Models

### Contacts
```sql
CREATE TABLE contacts (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  node_address TEXT UNIQUE NOT NULL,
  avatar_path TEXT,
  status TEXT DEFAULT 'offline',
  last_seen INTEGER,
  created_at INTEGER
);
```

### Messages
```sql
CREATE TABLE messages (
  id INTEGER PRIMARY KEY,
  conversation_id TEXT NOT NULL,
  sender_id TEXT NOT NULL,
  content TEXT NOT NULL,
  timestamp INTEGER NOT NULL,
  read_status INTEGER DEFAULT 0,
  has_attachment INTEGER DEFAULT 0,
  attachment_path TEXT
);
```

### Conversations
```sql
CREATE TABLE conversations (
  id TEXT PRIMARY KEY,
  type TEXT NOT NULL,  -- 'direct' or 'group'
  title TEXT,          -- null for direct conversations
  created_at INTEGER NOT NULL,
  last_message_at INTEGER
);
```

### Conversation Members
```sql
CREATE TABLE conversation_members (
  conversation_id TEXT NOT NULL,
  member_address TEXT NOT NULL,
  join_timestamp INTEGER NOT NULL,
  is_admin INTEGER DEFAULT 0,
  PRIMARY KEY (conversation_id, member_address)
);
```

## API Endpoints

### Contacts API
- `GET /api/contacts` - List all contacts
- `POST /api/contacts` - Add new contact
- `GET /api/contacts/:id` - Get contact details
- `PUT /api/contacts/:id` - Update contact
- `DELETE /api/contacts/:id` - Remove contact

### Messages API
- `GET /api/conversations` - List all conversations
- `GET /api/conversations/:id/messages` - Get messages for a conversation
- `POST /api/conversations/:id/messages` - Send a message to conversation
- `GET /api/messages/:id` - Get specific message
- `PUT /api/messages/:id/read` - Mark message as read

### Group Chat API
- `POST /api/conversations` - Create new conversation (direct or group)
- `PUT /api/conversations/:id` - Update conversation details (group name, etc.)
- `POST /api/conversations/:id/members` - Add member to conversation
- `DELETE /api/conversations/:id/members/:address` - Remove member

## Real-time Communication
- WebSocket connection for instant message delivery
- Notification of member status changes
- Typing indicators
- Message read receipts

## Storage Strategy
- **SQLite**: Primary storage for contacts, messages, and conversations
- **Key-Value Store**: Caching conversation status, online status
- **VFS**: Store message attachments and profile images

## UI Components
1. **ContactList**: Displays all contacts with online status
2. **ConversationList**: Shows recent conversations with preview
3. **MessageThread**: Displays messages for current conversation
4. **ComposeBox**: Interface for writing and sending messages
5. **GroupInfo**: Displays and manages group members

## Message Flow
1. User composes message in UI
2. Message sent to backend via API
3. Backend stores message in SQLite
4. Backend broadcasts message to recipients via WebSocket
5. Recipients' UI updates in real-time
6. Read receipts sent back via WebSocket

## Security Considerations
- Node address verification for contact addition
- Capability-based access for shared files
- Encrypted message content using node capabilities
- Validation of message sources

## Implementation Phases
1. **Phase 1**: Contact management and direct messaging
2. **Phase 2**: Group conversations
3. **Phase 3**: File sharing and attachments
4. **Phase 4**: Enhanced features (typing indicators, read receipts)