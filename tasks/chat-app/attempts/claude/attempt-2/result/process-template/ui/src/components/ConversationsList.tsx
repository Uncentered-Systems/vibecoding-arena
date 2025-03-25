import React from 'react';
import { Conversation } from '../types/types';

interface ConversationsListProps {
  conversations: Conversation[];
  selectedConversation: Conversation | null;
  onSelectConversation: (conversation: Conversation) => void;
}

const ConversationsList: React.FC<ConversationsListProps> = ({ 
  conversations, 
  selectedConversation, 
  onSelectConversation 
}) => {
  const formatTime = (timestamp: number | undefined | null) => {
    if (!timestamp) return '';
    
    const date = new Date(timestamp * 1000);
    return new Intl.DateTimeFormat('en-US', {
      hour: '2-digit',
      minute: '2-digit',
    }).format(date);
  };

  return (
    <div className="conversations-list">
      {conversations.length === 0 ? (
        <div className="empty-list">No conversations available</div>
      ) : (
        <ul>
          {conversations.map((conversation) => (
            <li 
              key={conversation.id}
              className={`conversation-item ${selectedConversation?.id === conversation.id ? 'selected' : ''}`}
              onClick={() => onSelectConversation(conversation)}
            >
              <div className="conversation-info">
                <div className="conversation-title">
                  {conversation.title || (conversation.type_ === 'direct' ? 'Direct Message' : 'Group Chat')}
                  {conversation.unread_count > 0 && (
                    <span className="unread-badge">{conversation.unread_count}</span>
                  )}
                </div>
                <div className="conversation-preview">
                  {conversation.last_message ? (
                    <span className="last-message">{conversation.last_message}</span>
                  ) : (
                    <span className="no-messages">No messages</span>
                  )}
                </div>
              </div>
              {conversation.last_message_at && (
                <div className="conversation-time">
                  {formatTime(conversation.last_message_at)}
                </div>
              )}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
};

export default ConversationsList;