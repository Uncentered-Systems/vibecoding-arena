import { useState, useEffect, useCallback, FormEvent, ChangeEvent } from "react";
import HyperwareClientApi from "@hyperware-ai/client-api";
import "./App.css";
import { 
  SendHyperwareChatMessage, 
  NewGroupMessage, 
  Group, 
  CreateGroupRequest,
  GroupMemberRequest,
  SendGroupMessageRequest,
  Contact
} from "./types/HyperwareChat";
import useHyperwareChatStore from "./store/hyperware_chat";

// Use the full process path
const PROCESS_PATH = "hyperware-chat:hyperware-chat:template.os";
const BASE_URL = `/${PROCESS_PATH}`;
if (window.our) window.our.process = PROCESS_PATH;

const PROXY_TARGET = import.meta.env.VITE_NODE_URL || "http://localhost:8080";

// This env also has BASE_URL which should match the process + package name
const WEBSOCKET_URL = import.meta.env.DEV
  ? `${PROXY_TARGET.replace('http', 'ws')}`
  : undefined;

function formatDate(timestamp: number | undefined): string {
  if (!timestamp) return '';
  return new Date(timestamp * 1000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
}

function App() {
  const { 
    hyperware_chats, 
    addMessage, 
    set, 
    groups, 
    groupMessages, 
    addGroup, 
    addGroupMessage, 
    contacts,
    addContact,
    removeContact,
    selectedChatId,
    selectedGroupId,
    setSelectedChatId,
    setSelectedGroupId
  } = useHyperwareChatStore();

  // Form states
  const [target, setTarget] = useState("");
  const [message, setMessage] = useState("");
  const [nodeConnected, setNodeConnected] = useState(true);
  const [api, setApi] = useState<HyperwareClientApi | undefined>();
  
  // UI state
  const [activeTab, setActiveTab] = useState<'chats' | 'groups' | 'contacts'>('chats');
  
  // Group form state
  const [showGroupForm, setShowGroupForm] = useState(false);
  const [newGroupName, setNewGroupName] = useState("");
  const [selectedContacts, setSelectedContacts] = useState<string[]>([]);
  
  // Contact form state
  const [showContactForm, setShowContactForm] = useState(false);
  const [newContactId, setNewContactId] = useState("");

  useEffect(() => {
    // Connect to the WebSocket first
    console.log('WEBSOCKET URL', WEBSOCKET_URL);
    
    // Once connected, we'll request data via WebSocket
    const initializeData = async (api: HyperwareClientApi) => {
      // Request message history via WebSocket
      try {
        const wsMessage1: any = {};
        wsMessage1.data = JSON.stringify({ GetMessages: {} });
        api.send(wsMessage1);
        console.log("Requested message history via WebSocket");
        
        // Request contacts
        const wsMessage2: any = {};
        wsMessage2.data = JSON.stringify({ GetContacts: {} });
        api.send(wsMessage2);
        console.log("Requested contacts via WebSocket");
        
        // Request groups
        const wsMessage3: any = {};
        wsMessage3.data = JSON.stringify({ GetGroups: {} });
        api.send(wsMessage3);
        console.log("Requested groups via WebSocket");
      } catch (error) {
        console.error("Error sending data requests via WebSocket:", error);
      }
    };

    if (window.our?.node && window.our?.process) {
      const api = new HyperwareClientApi({
        uri: WEBSOCKET_URL,
        nodeId: window.our.node,
        processId: window.our.process,
        onOpen: (_event, apiInstance) => {
          console.log("Connected to Hyperware");
          // Initialize data once connected
          initializeData(apiInstance);
        },
        onMessage: (json, _api) => {
          console.log('WEBSOCKET MESSAGE', json)
          try {
            const data = JSON.parse(json);
            console.log("WebSocket received message", data);
            const [messageType] = Object.keys(data);
            if (!messageType) return;

            // Handle different message types
            switch (messageType) {
              case "NewMessage":
                addMessage(data.NewMessage);
                break;
              case "NewGroupMessage":
                addGroupMessage(data.NewGroupMessage);
                break;
              case "NewGroup":
                // Replace any temporary group with the same name
                const newGroup = data.NewGroup;
                console.log("Received new group:", newGroup);
                
                // Remove any temporary groups
                const updatedGroups = groups.filter(g => !g.id.startsWith('temp_') || g.name !== newGroup.name);
                // Add the new group
                set({ groups: [...updatedGroups, newGroup] });
                // Set selected group to the new one
                setSelectedGroupId(newGroup.id);
                break;
              case "ContactAdded":
                console.log("Contact added:", data.ContactAdded);
                // Check if already exists
                const contactExists = contacts.some(c => c.id === data.ContactAdded.id);
                if (!contactExists) {
                  addContact(data.ContactAdded);
                }
                break;
              case "Messages":
                console.log("Received message history:", data.Messages);
                // Update message history
                set({ hyperware_chats: { ...(data.Messages || {}), "New Chat": [] } });
                break;
              case "Contacts":
                console.log("Received contacts:", data.Contacts);
                if (data.Contacts) {
                  set({ contacts: data.Contacts });
                }
                break;
              case "Groups":
                console.log("Received groups:", data.Groups);
                if (data.Groups) {
                  set({ groups: data.Groups });
                }
                break;
              case "GroupMessages":
                console.log("Received group messages:", data.GroupMessages);
                if (data.GroupMessages && data.GroupMessages.group_id) {
                  const { group_id, messages } = data.GroupMessages;
                  const newGroupMessages = { ...groupMessages };
                  newGroupMessages[group_id] = messages;
                  set({ groupMessages: newGroupMessages });
                }
                break;
              default:
                console.log("Unknown message type:", messageType);
            }
          } catch (error) {
            console.error("Error parsing WebSocket message", error);
          }
        },
      });

      setApi(api);
    } else {
      setNodeConnected(false);
    }
  }, [groups, contacts, groupMessages, addMessage, addGroupMessage, set, setSelectedGroupId, addContact]);

  const startChat = useCallback(
    (event: FormEvent) => {
      event.preventDefault();

      if (!target) return;

      const newHyperwareChats = { ...hyperware_chats };
      newHyperwareChats[target] = [];

      setSelectedChatId(target);
      set({ hyperware_chats: newHyperwareChats });

      setTarget("");
    },
    [hyperware_chats, set, target]
  );

  const sendMessage = useCallback(
    async (event: FormEvent) => {
      event.preventDefault();

      if (!message || !api) return;

      try {
        // If a direct chat is selected
        if (selectedChatId && selectedChatId !== "New Chat") {
          // Create a message object for direct chat
          const data = {
            Send: {
              target: selectedChatId,
              message,
            },
          } as SendHyperwareChatMessage;

          // Send via WebSocket
          console.log("Sending message via WebSocket:", data);
          
          // Build WebSocket message dynamically
          const wsMessage: any = {};
          wsMessage.data = JSON.stringify(data);
          
          api.send(wsMessage);
          console.log("Sent message via WebSocket");
          
          setMessage("");
        } 
        // If a group is selected
        else if (selectedGroupId) {
          // Create a message object for group chat
          const wsData = {
            GroupMessage: {
              group_id: selectedGroupId,
              message,
            }
          };

          // Send via WebSocket
          console.log("Sending group message via WebSocket:", wsData);
          
          // Build WebSocket message dynamically
          const wsMessage: any = {};
          wsMessage.data = JSON.stringify(wsData);
          
          api.send(wsMessage);
          console.log("Sent group message via WebSocket");
          
          setMessage("");
        }
      } catch (error) {
        console.error("Error sending message via WebSocket:", error);
      }
    },
    [message, selectedChatId, selectedGroupId, api]
  );

  const handleCreateGroup = async (e: FormEvent) => {
    e.preventDefault();
    
    if (!newGroupName || selectedContacts.length === 0 || !api) {
      alert("Group name and at least one contact are required");
      return;
    }
    
    try {
      const wsData = {
        CreateGroup: {
          name: newGroupName,
          members: selectedContacts
        }
      };
      
      // Send via WebSocket
      console.log("Creating group via WebSocket:", wsData);
      
      // Build WebSocket message dynamically
      const wsMessage: any = {};
      wsMessage.data = JSON.stringify(wsData);
      
      api.send(wsMessage);
      console.log("Group creation request sent");
      
      // Since we don't get an immediate response from WebSocket, we'll create a temporary group
      // The real data will be updated when we receive the confirmation via WebSocket
      const tempId = `temp_${Date.now()}`;
      const tempGroup: Group = {
        id: tempId,
        name: newGroupName,
        members: selectedContacts,
        created_by: window.our?.node || "",
        created_at: Math.floor(Date.now() / 1000)
      };
      
      // Add the temporary group
      addGroup(tempGroup);
      
      // Reset form
      setNewGroupName("");
      setSelectedContacts([]);
      setShowGroupForm(false);
      
      // Switch to the new group
      setSelectedGroupId(tempId);
      setActiveTab('groups');
    } catch (error) {
      console.error("Error creating group via WebSocket:", error);
      alert("Failed to create group. See console for details.");
    }
  };

  const handleAddContact = async (e: FormEvent) => {
    e.preventDefault();
    
    if (!newContactId || !api) {
      alert("Contact ID is required");
      return;
    }
    
    try {
      const wsData = {
        AddContact: {
          id: newContactId,
          name: newContactId // For simplicity, using the ID as the name
        }
      };
      
      // Send via WebSocket
      console.log("Adding contact via WebSocket:", wsData);
      
      // Build WebSocket message dynamically
      const wsMessage: any = {};
      wsMessage.data = JSON.stringify(wsData);
      
      api.send(wsMessage);
      console.log("Contact add request sent");
      
      // Add the contact to our local store
      const newContact: Contact = {
        id: newContactId,
        name: newContactId // For simplicity, using the ID as the name
      };
      
      addContact(newContact);
      setNewContactId("");
      setShowContactForm(false);
    } catch (error) {
      console.error("Error adding contact via WebSocket:", error);
      alert("Failed to add contact. See console for details.");
    }
  };

  const renderChatTab = () => {
    return (
      <div className="tab-content">
        <div className="sidebar">
          <div className="sidebar-header">
            <h3>Direct Chats</h3>
            <button onClick={() => setSelectedChatId("New Chat")} className="icon-button">+</button>
          </div>
          <ul className="chat-list">
            {Object.keys(hyperware_chats)
              .filter(chatId => chatId !== "New Chat")
              .map((chatId) => (
                <li key={chatId} className={selectedChatId === chatId ? 'active' : ''}>
                  <button onClick={() => setSelectedChatId(chatId)}>
                    {chatId}
                  </button>
                </li>
              ))}
          </ul>
        </div>
        
        <div className="chat-area">
          {selectedChatId === "New Chat" ? (
            <div className="new-chat-form">
              <h3>Start a New Chat</h3>
              <form onSubmit={startChat}>
                <div className="form-group">
                  <label htmlFor="target">Node ID</label>
                  <input
                    type="text"
                    id="target"
                    value={target}
                    onChange={(e) => setTarget(e.target.value)}
                    placeholder="Enter node ID"
                    required
                  />
                </div>
                <button type="submit" className="primary-button">Start Chat</button>
              </form>
            </div>
          ) : (
            <div className="chat-container">
              <div className="chat-header">
                <h3>{selectedChatId}</h3>
              </div>
              
              <div className="messages-container">
                <ul className="message-list">
                  {selectedChatId &&
                    hyperware_chats[selectedChatId]?.map((message, index) => (
                      <li key={index} className={`message ${message.author === window.our?.node ? 'ours' : ''}`}>
                        <div className="message-content">{message.content}</div>
                        <div className="message-time">{formatDate(message.timestamp)}</div>
                      </li>
                    ))}
                </ul>
              </div>
              
              <form onSubmit={sendMessage} className="message-form">
                <input
                  type="text"
                  placeholder="Type a message..."
                  value={message}
                  onChange={(e) => setMessage(e.target.value)}
                />
                <button type="submit">Send</button>
              </form>
            </div>
          )}
        </div>
      </div>
    );
  };

  const renderGroupsTab = () => {
    return (
      <div className="tab-content">
        <div className="sidebar">
          <div className="sidebar-header">
            <h3>Group Chats</h3>
            <button onClick={() => setShowGroupForm(true)} className="icon-button">+</button>
          </div>
          <ul className="chat-list">
            {groups.map((group) => (
              <li key={group.id} className={selectedGroupId === group.id ? 'active' : ''}>
                <button onClick={() => setSelectedGroupId(group.id)}>
                  {group.name}
                </button>
              </li>
            ))}
          </ul>
        </div>
        
        <div className="chat-area">
          {showGroupForm ? (
            <div className="new-group-form">
              <h3>Create a New Group</h3>
              <form onSubmit={handleCreateGroup}>
                <div className="form-group">
                  <label htmlFor="group-name">Group Name</label>
                  <input
                    type="text"
                    id="group-name"
                    value={newGroupName}
                    onChange={(e) => setNewGroupName(e.target.value)}
                    placeholder="Enter group name"
                    required
                  />
                </div>
                
                <div className="form-group">
                  <label>Select Contacts</label>
                  <div className="contact-selection">
                    {contacts.length === 0 ? (
                      <p>No contacts available. Add contacts first.</p>
                    ) : (
                      contacts.map(contact => (
                        <div key={contact.id} className="contact-checkbox">
                          <input
                            type="checkbox" 
                            id={`contact-${contact.id}`}
                            checked={selectedContacts.includes(contact.id)}
                            onChange={(e) => {
                              if (e.target.checked) {
                                setSelectedContacts([...selectedContacts, contact.id]);
                              } else {
                                setSelectedContacts(selectedContacts.filter(id => id !== contact.id));
                              }
                            }}
                          />
                          <label htmlFor={`contact-${contact.id}`}>{contact.name || contact.id}</label>
                        </div>
                      ))
                    )}
                  </div>
                </div>
                
                <div className="button-row">
                  <button type="button" onClick={() => setShowGroupForm(false)}>Cancel</button>
                  <button type="submit" className="primary-button" disabled={contacts.length === 0}>Create Group</button>
                </div>
              </form>
            </div>
          ) : selectedGroupId ? (
            <div className="chat-container">
              <div className="chat-header">
                <h3>{groups.find(g => g.id === selectedGroupId)?.name || selectedGroupId}</h3>
                <div className="group-members">
                  {groups.find(g => g.id === selectedGroupId)?.members.length || 0} members
                </div>
              </div>
              
              <div className="messages-container">
                <ul className="message-list">
                  {groupMessages[selectedGroupId]?.map((message, index) => (
                    <li key={index} className={`message ${message.author === window.our?.node ? 'ours' : ''}`}>
                      <div className="message-author">{message.author}</div>
                      <div className="message-content">{message.content}</div>
                      <div className="message-time">{formatDate(message.timestamp)}</div>
                    </li>
                  ))}
                </ul>
              </div>
              
              <form onSubmit={sendMessage} className="message-form">
                <input
                  type="text"
                  placeholder="Type a message..."
                  value={message}
                  onChange={(e) => setMessage(e.target.value)}
                />
                <button type="submit">Send</button>
              </form>
            </div>
          ) : (
            <div className="empty-state">
              <p>Select a group chat or create a new one</p>
            </div>
          )}
        </div>
      </div>
    );
  };

  const renderContactsTab = () => {
    return (
      <div className="tab-content">
        <div className="contacts-container">
          <div className="contacts-header">
            <h3>Contacts</h3>
            <button onClick={() => setShowContactForm(true)} className="icon-button">+</button>
          </div>
          
          {showContactForm ? (
            <div className="new-contact-form">
              <h4>Add New Contact</h4>
              <form onSubmit={handleAddContact}>
                <div className="form-group">
                  <label htmlFor="contact-id">Contact Node ID</label>
                  <input
                    type="text"
                    id="contact-id"
                    value={newContactId}
                    onChange={(e) => setNewContactId(e.target.value)}
                    placeholder="Enter node ID"
                    required
                  />
                </div>
                <div className="button-row">
                  <button type="button" onClick={() => setShowContactForm(false)}>Cancel</button>
                  <button type="submit" className="primary-button">Add Contact</button>
                </div>
              </form>
            </div>
          ) : (
            <div className="contacts-list">
              {contacts.length === 0 ? (
                <div className="empty-state">
                  <p>No contacts yet. Add some contacts to start chatting.</p>
                </div>
              ) : (
                <ul>
                  {contacts.map(contact => (
                    <li key={contact.id} className="contact-item">
                      <div className="contact-info">
                        <span className="contact-name">{contact.name || contact.id}</span>
                        <span className="contact-id">{contact.id}</span>
                      </div>
                      <div className="contact-actions">
                        <button 
                          onClick={() => {
                            // Check if chat already exists
                            if (!hyperware_chats[contact.id]) {
                              const newHyperwareChats = { ...hyperware_chats };
                              newHyperwareChats[contact.id] = [];
                              set({ hyperware_chats: newHyperwareChats });
                            }
                            setSelectedChatId(contact.id);
                            setActiveTab('chats');
                          }}
                          className="action-button"
                        >
                          Chat
                        </button>
                        <button 
                          onClick={() => {
                            if (window.confirm(`Remove ${contact.name || contact.id} from contacts?`)) {
                              removeContact(contact.id);
                            }
                          }}
                          className="action-button danger"
                        >
                          Remove
                        </button>
                      </div>
                    </li>
                  ))}
                </ul>
              )}
            </div>
          )}
        </div>
      </div>
    );
  };

  return (
    <div className="app-container">
      <header className="app-header">
        <h1>Hyperware Chat</h1>
        <div className="user-info">
          Your ID: <strong>{window.our?.node}</strong>
        </div>
      </header>
      
      {!nodeConnected && (
        <div className="node-not-connected">
          <h2>Node not connected</h2>
          <p>
            You need to start a node at {PROXY_TARGET} before you can use this UI
            in development.
          </p>
        </div>
      )}
      
      <main className="main-content">
        <div className="tabs">
          <button 
            className={activeTab === 'chats' ? 'active' : ''} 
            onClick={() => setActiveTab('chats')}
          >
            Chats
          </button>
          <button 
            className={activeTab === 'groups' ? 'active' : ''} 
            onClick={() => setActiveTab('groups')}
          >
            Groups
          </button>
          <button 
            className={activeTab === 'contacts' ? 'active' : ''} 
            onClick={() => setActiveTab('contacts')}
          >
            Contacts
          </button>
        </div>
        
        <div className="tab-panels">
          {activeTab === 'chats' && renderChatTab()}
          {activeTab === 'groups' && renderGroupsTab()}
          {activeTab === 'contacts' && renderContactsTab()}
        </div>
      </main>
    </div>
  );
}

export default App;
