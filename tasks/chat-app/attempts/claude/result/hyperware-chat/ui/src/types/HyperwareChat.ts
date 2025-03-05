export interface HyperwareChatMessage {
  author: string
  content: string
  timestamp?: number  // Optional for backward compatibility
}

export interface NewMessage {
  hyperware_chat: string
  author: string
  content: string
  timestamp: number
}

export interface NewGroupMessage {
  group_id: string
  author: string
  content: string
  timestamp: number
}

export interface SendHyperwareChatMessage {
  Send: {
    target: string
    message: string
  }
}

export interface Contact {
  id: string
  name: string
}

export interface Group {
  id: string
  name: string
  members: string[]
  created_by: string
  created_at: number
}

export interface CreateGroupRequest {
  name: string
  members: string[]
}

export interface GroupMemberRequest {
  group_id: string
  member: string
}

export interface SendGroupMessageRequest {
  group_id: string
  message: string
}

// API response type
export interface ApiResponse<T> {
  success: boolean
  data?: T
  error?: string
}

// HyperwareChats consists of a map of counterparty to an array of messages
export interface HyperwareChats {
  [counterparty: string]: HyperwareChatMessage[]
}

// Group messages by group ID
export interface GroupMessages {
  [groupId: string]: HyperwareChatMessage[]
}
