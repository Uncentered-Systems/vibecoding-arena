// API Response types
export type ApiResponse = {
  // Base responses
  status?: {
    connected_clients: number;
    message_count: number;
    message_counts_by_channel: Record<string, number>;
  };
  success?: {
    message: string;
  };
  error?: {
    code: number;
    message: string;
  };
  
  // Data responses
  contacts?: Contact[];
  contact?: Contact;
  conversations?: Conversation[];
  conversation_detail?: {
    conversation: Conversation;
    members: ConversationMember[];
  };
  messages?: ChatMessage[];
};

// Contact model
export interface Contact {
  id: number | null;
  name: string;
  node_address: string;
  avatar_path?: string | null;
  status: string;
  last_seen?: number | null;
  created_at: number;
}

// Conversation model
export interface Conversation {
  id: string;
  type_: string; // "direct" or "group"
  title?: string | null;
  created_at: number;
  last_message_at?: number | null;
  last_message?: string | null;
  unread_count: number;
}

// Conversation member
export interface ConversationMember {
  conversation_id: string;
  member_address: string;
  join_timestamp: number;
  is_admin: boolean;
  name?: string | null;
}

// Chat message
export interface ChatMessage {
  id: number | null;
  conversation_id: string;
  sender_id: string;
  content: string;
  timestamp: number;
  read_status: number;
  has_attachment: boolean;
  attachment_path?: string | null;
}

// WebSocket message types
export type WebSocketMessage = 
  | { type: 'Auth'; node_address: string }
  | { type: 'ChatMessage'; conversation_id: string; sender_id: string; content: string; timestamp: number; has_attachment: boolean; attachment_path?: string | null }
  | { type: 'TypingIndicator'; conversation_id: string; user_id: string; is_typing: boolean }
  | { type: 'ReadReceipt'; message_ids: number[] }
  | { type: 'StatusChange'; node_address: string; status: string };