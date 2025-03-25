import React, { useRef, useEffect } from 'react';
import { Conversation, ChatMessage } from '../types/types';

interface ChatWindowProps {
  conversation: Conversation;
  messages: ChatMessage[];
  currentUser: string;
  newMessage: string;
  setNewMessage: (message: string) => void;
  onSendMessage: () => void;
}

const ChatWindow: React.FC<ChatWindowProps> = ({
  conversation,
  messages,
  currentUser,
  newMessage,
  setNewMessage,
  onSendMessage,
}) => {
  const messagesEndRef = useRef<HTMLDivElement>(null);

  // Scroll to bottom when messages change
  useEffect(() => {
    if (messagesEndRef.current) {
      messagesEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [messages]);

  const formatTimestamp = (timestamp: number) => {
    const date = new Date(timestamp * 1000);
    return new Intl.DateTimeFormat('en-US', {
      hour: '2-digit',
      minute: '2-digit',
    }).format(date);
  };

  const handleKeyPress = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      onSendMessage();
    }
  };

  return (
    <div className="chat-window">
      <div className="chat-header">
        <h2>{conversation.title || (conversation.type_ === 'direct' ? 'Direct Chat' : 'Group Chat')}</h2>
      </div>

      <div className="messages-container">
        {messages.length === 0 ? (
          <div className="no-messages">No messages yet. Start a conversation!</div>
        ) : (
          messages.map((message, index) => (
            <div
              key={index}
              className={`message ${message.sender_id === currentUser ? 'sent' : 'received'}`}
            >
              <div className="message-content">
                {message.content}
                {message.has_attachment && message.attachment_path && (
                  <div className="attachment">
                    <a href={message.attachment_path} target="_blank" rel="noopener noreferrer">
                      Attachment
                    </a>
                  </div>
                )}
              </div>
              <div className="message-meta">
                <span className="message-time">{formatTimestamp(message.timestamp)}</span>
                {message.read_status === 1 && message.sender_id === currentUser && (
                  <span className="read-status">Read</span>
                )}
              </div>
            </div>
          ))
        )}
        <div ref={messagesEndRef}></div>
      </div>

      <div className="message-input">
        <input
          type="text"
          placeholder="Type a message..."
          value={newMessage}
          onChange={(e) => setNewMessage(e.target.value)}
          onKeyPress={handleKeyPress}
        />
        <button onClick={onSendMessage} disabled={!newMessage.trim()}>
          Send
        </button>
      </div>
    </div>
  );
};

export default ChatWindow;