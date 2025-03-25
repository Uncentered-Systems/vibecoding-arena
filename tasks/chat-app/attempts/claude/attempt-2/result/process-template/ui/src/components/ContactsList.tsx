import React from 'react';
import { Contact } from '../types/types';

interface ContactsListProps {
  contacts: Contact[];
  selectedContact: Contact | null;
  onSelectContact: (contact: Contact) => void;
}

const ContactsList: React.FC<ContactsListProps> = ({ 
  contacts, 
  selectedContact, 
  onSelectContact 
}) => {
  return (
    <div className="contacts-list">
      {contacts.length === 0 ? (
        <div className="empty-list">No contacts available</div>
      ) : (
        <ul>
          {contacts.map((contact) => (
            <li 
              key={contact.node_address}
              className={`contact-item ${selectedContact?.id === contact.id ? 'selected' : ''}`}
              onClick={() => onSelectContact(contact)}
            >
              <div className="contact-avatar">
                {contact.avatar_path ? (
                  <img src={contact.avatar_path} alt={contact.name} />
                ) : (
                  <div className="default-avatar">{contact.name.charAt(0)}</div>
                )}
                <span className={`status-indicator ${contact.status}`} />
              </div>
              <div className="contact-info">
                <div className="contact-name">{contact.name}</div>
                <div className="contact-address">{contact.node_address}</div>
              </div>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
};

export default ContactsList;