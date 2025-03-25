import { useEffect, useState } from 'react';
import './App.css';
import { fetchStatus, fetchContacts, fetchConversations, fetchMessages, sendMessage, addContact, createConversation } from './utilities/api';
import { setupWebSocket } from './utilities/websocket';
import { Contact, Conversation, ChatMessage } from './types/types';
import ContactsList from './components/ContactsList';
import ConversationsList from './components/ConversationsList';
import ChatWindow from './components/ChatWindow';

function App() {
  // Use an effect to get our node address from Hyperware instead of hardcoding
  const [nodeAddress, setNodeAddress] = useState<string>('');
  const [contacts, setContacts] = useState<Contact[]>([]);
  const [conversations, setConversations] = useState<Conversation[]>([]);
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [selectedContact, setSelectedContact] = useState<Contact | null>(null);
  const [selectedConversation, setSelectedConversation] = useState<Conversation | null>(null);
  const [newMessage, setNewMessage] = useState<string>('');
  const [newContactName, setNewContactName] = useState<string>('');
  const [newContactAddress, setNewContactAddress] = useState<string>('');
  const [socket, setSocket] = useState<WebSocket | null>(null);
  const [connected, setConnected] = useState<boolean>(false);
  
  // Get our node address from Hyperware
  useEffect(() => {
    // In Hyperware, we can use the 'our.js' file to get our node info
    // The global 'our' object is added to window by Hyperware
    if (window.our && window.our.address) {
      // Format: node@app.publish.domain
      const addr = window.our.address;
      console.log('Got node address from Hyperware:', addr);
      setNodeAddress(addr);
    } else {
      console.warn('Could not get node address from Hyperware, using fallback');
      setNodeAddress(`node-${Math.floor(Math.random() * 10000)}.example.os`);
    }
  }, []);

  // Initialize WebSocket connection
  useEffect(() => {
    // Only connect when we have a valid node address
    if (!nodeAddress) {
      return;
    }

    console.log('Initializing WebSocket connection with node address:', nodeAddress);
    
    const ws = setupWebSocket({
      onOpen: () => {
        console.log('WebSocket connected');
        setConnected(true);
        
        // Authenticate with node address
        // Match the expected WebSocketMessage::Auth format
        ws.send(JSON.stringify({
          type: 'Auth',
          node_address: nodeAddress
        }));
      },
      onMessage: (data) => {
        console.log('WebSocket message:', data);
        
        // Handle different message types
        if (data.type === 'welcome' || data.type === 'auth_success') {
          console.log('Successfully connected to WebSocket server:', data);
        }
        else if (data.type === 'ChatMessage' && data.conversation_id === selectedConversation?.id) {
          // Add new message to the list
          setMessages(prev => [...prev, {
            id: null, // ID will be assigned by server
            conversation_id: data.conversation_id,
            sender_id: data.sender_id,
            content: data.content,
            timestamp: data.timestamp,
            read_status: 0,
            has_attachment: data.has_attachment || false,
            attachment_path: data.attachment_path
          }]);
        } else if (data.type === 'status_change' || data.type === 'StatusChange') {
          // Update contact status
          setContacts(prev => prev.map(contact => 
            contact.node_address === data.node_address 
              ? { ...contact, status: data.status } 
              : contact
          ));
        }
      },
      onClose: () => {
        console.log('WebSocket disconnected');
        setConnected(false);
      }
    });
    
    setSocket(ws);
    
    return () => {
      if (ws) {
        ws.close();
      }
    };
  }, [nodeAddress]); // Re-initialize when node address changes

  // Load initial data
  useEffect(() => {
    // Only load data when we have a valid node address
    if (!nodeAddress) {
      return;
    }
    
    const loadInitialData = async () => {
      try {
        console.log('Loading initial data for node:', nodeAddress);
        
        // Get status
        const status = await fetchStatus();
        console.log('Status:', status);
        
        // Get contacts
        const contactsData = await fetchContacts();
        setContacts(contactsData);
        
        // Get conversations
        const conversationsData = await fetchConversations(nodeAddress);
        setConversations(conversationsData);
      } catch (error) {
        console.error('Error loading initial data:', error);
      }
    };
    
    loadInitialData();
  }, [nodeAddress]);

  // Load messages when conversation is selected
  useEffect(() => {
    if (selectedConversation) {
      fetchMessages(selectedConversation.id)
        .then(messagesData => {
          setMessages(messagesData);
        })
        .catch(error => {
          console.error('Error loading messages:', error);
        });
    } else {
      setMessages([]);
    }
  }, [selectedConversation]);

  const handleContactSelect = (contact: Contact) => {
    setSelectedContact(contact);
    
    // Find or create a direct conversation with this contact
    const existingConversation = conversations.find(conv => 
      conv.type_ === 'direct' && 
      conv.id.includes(contact.node_address) && 
      conv.id.includes(nodeAddress)
    );
    
    if (existingConversation) {
      setSelectedConversation(existingConversation);
    } else {
      // Create a new conversation
      createConversation({
        title: null,
        members: [nodeAddress, contact.node_address],
        is_group: false
      })
        .then(response => {
          // Refresh conversations
          return fetchConversations(nodeAddress);
        })
        .then(conversationsData => {
          setConversations(conversationsData);
          // Find the newly created conversation
          const newConversation = conversationsData.find(conv => 
            conv.type_ === 'direct' && 
            conv.id.includes(contact.node_address) && 
            conv.id.includes(nodeAddress)
          );
          if (newConversation) {
            setSelectedConversation(newConversation);
          }
        })
        .catch(error => {
          console.error('Error creating conversation:', error);
        });
    }
  };

  const handleConversationSelect = (conversation: Conversation) => {
    setSelectedConversation(conversation);
    setSelectedContact(null);
  };

  const handleSendMessage = () => {
    if (!selectedConversation || !newMessage.trim()) return;
    
    sendMessage({
      conversation_id: selectedConversation.id,
      sender_id: nodeAddress,
      content: newMessage,
      has_attachment: false,
      attachment_path: null
    })
      .then(response => {
        // Add message to the list
        setMessages(prev => [...prev, {
          id: null, // Will be assigned by server
          conversation_id: selectedConversation.id,
          sender_id: nodeAddress,
          content: newMessage,
          timestamp: Date.now() / 1000,
          read_status: 0,
          has_attachment: false,
          attachment_path: null
        }]);
        
        // Clear input
        setNewMessage('');
        
        // Refresh conversations to update last message
        return fetchConversations(nodeAddress);
      })
      .then(conversationsData => {
        setConversations(conversationsData);
      })
      .catch(error => {
        console.error('Error sending message:', error);
      });
  };

  const handleAddContact = () => {
    if (!newContactName.trim() || !newContactAddress.trim()) return;
    
    addContact({
      name: newContactName,
      node_address: newContactAddress
    })
      .then(response => {
        // Refresh contacts
        return fetchContacts();
      })
      .then(contactsData => {
        setContacts(contactsData);
        // Clear inputs
        setNewContactName('');
        setNewContactAddress('');
      })
      .catch(error => {
        console.error('Error adding contact:', error);
      });
  };

  return (
    <div className="app-container">
      <header className="app-header">
        <h1>Hyperchat</h1>
        <div className="user-info">
          <span>Your Node: {nodeAddress}</span>
          <span className={`status-indicator ${connected ? 'online' : 'offline'}`}>
            {connected ? 'Connected' : 'Disconnected'}
          </span>
        </div>
      </header>
      
      <div className="main-container">
        <div className="sidebar">
          <div className="add-contact-form">
            <h3>Add Contact</h3>
            <input
              type="text"
              placeholder="Name"
              value={newContactName}
              onChange={(e) => setNewContactName(e.target.value)}
            />
            <input
              type="text"
              placeholder="Node Address"
              value={newContactAddress}
              onChange={(e) => setNewContactAddress(e.target.value)}
            />
            <button onClick={handleAddContact}>Add</button>
          </div>
          
          <div className="contacts-container">
            <h3>Contacts</h3>
            <ContactsList 
              contacts={contacts} 
              onSelectContact={handleContactSelect} 
              selectedContact={selectedContact}
            />
          </div>
          
          <div className="conversations-container">
            <h3>Conversations</h3>
            <ConversationsList 
              conversations={conversations} 
              onSelectConversation={handleConversationSelect} 
              selectedConversation={selectedConversation}
            />
          </div>
        </div>
        
        <div className="chat-container">
          {selectedConversation ? (
            <ChatWindow
              conversation={selectedConversation}
              messages={messages}
              currentUser={nodeAddress}
              onSendMessage={handleSendMessage}
              newMessage={newMessage}
              setNewMessage={setNewMessage}
            />
          ) : (
            <div className="empty-chat">
              <p>Select a contact or conversation to start chatting</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

export default App;