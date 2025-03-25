import { ApiResponse, Contact, Conversation, ChatMessage } from '../types/types';

// Use the base URL from the Vite config
// This will properly prepend process name and package info for Hyperware
const API_BASE_URL = `${import.meta.env.BASE_URL}/api`;

// Helper function for making API requests
async function apiRequest<T>(
  endpoint: string,
  method: string = 'GET',
  body?: any
): Promise<T> {
  const options: RequestInit = {
    method,
    headers: {
      'Content-Type': 'application/json',
    },
  };

  if (body) {
    options.body = JSON.stringify(body);
  }

  const response = await fetch(`${API_BASE_URL}${endpoint}`, options);
  
  if (!response.ok) {
    const errorData = await response.json();
    throw new Error(errorData.message || 'API request failed');
  }
  
  return response.json();
}

// Fetch application status
export async function fetchStatus(): Promise<any> {
  const response = await apiRequest<ApiResponse>('/status');
  return response;
}

// Contact Management
export async function fetchContacts(): Promise<Contact[]> {
  const response = await apiRequest<ApiResponse>('/contacts');
  
  if ('contacts' in response) {
    return response.contacts;
  }
  
  return [];
}

export async function addContact(contact: { name: string; node_address: string }): Promise<ApiResponse> {
  return apiRequest<ApiResponse>('/contacts', 'POST', contact);
}

export async function deleteContact(id: number): Promise<ApiResponse> {
  return apiRequest<ApiResponse>(`/contacts/${id}`, 'DELETE');
}

// Conversation Management
export async function fetchConversations(userAddress: string): Promise<Conversation[]> {
  const response = await apiRequest<ApiResponse>(`/conversations?user=${userAddress}`);
  
  if ('conversations' in response) {
    return response.conversations;
  }
  
  return [];
}

export async function createConversation(data: { 
  title: string | null; 
  members: string[]; 
  is_group: boolean 
}): Promise<ApiResponse> {
  return apiRequest<ApiResponse>('/conversations', 'POST', data);
}

export async function addConversationMember(
  conversationId: string,
  nodeAddress: string
): Promise<ApiResponse> {
  return apiRequest<ApiResponse>(
    `/conversations/${conversationId}/members`,
    'POST',
    { node_address: nodeAddress }
  );
}

// Message Management
export async function fetchMessages(conversationId: string): Promise<ChatMessage[]> {
  const response = await apiRequest<ApiResponse>(`/conversations/${conversationId}/messages`);
  
  if ('messages' in response) {
    return response.messages;
  }
  
  return [];
}

export async function sendMessage(data: {
  conversation_id: string;
  sender_id: string;
  content: string;
  has_attachment: boolean;
  attachment_path: string | null;
}): Promise<ApiResponse> {
  return apiRequest<ApiResponse>(
    `/conversations/${data.conversation_id}/messages`,
    'POST',
    data
  );
}

export async function markMessageRead(messageId: number): Promise<ApiResponse> {
  return apiRequest<ApiResponse>(`/messages/${messageId}/read`, 'PUT');
}