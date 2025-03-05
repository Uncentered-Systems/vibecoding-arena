import { create } from 'zustand'
import { 
  NewMessage, 
  NewGroupMessage, 
  HyperwareChats, 
  GroupMessages, 
  Group,
  Contact
} from '../types/HyperwareChat'
import { persist, createJSONStorage } from 'zustand/middleware'

export interface HyperwareChatStore {
  // Direct messages
  hyperware_chats: HyperwareChats
  addMessage: (msg: NewMessage) => void
  
  // Group chats
  groups: Group[]
  groupMessages: GroupMessages
  addGroupMessage: (msg: NewGroupMessage) => void
  addGroup: (group: Group) => void
  
  // Contacts
  contacts: Contact[]
  addContact: (contact: Contact) => void
  removeContact: (contactId: string) => void
  
  // Selected chat/group state
  selectedChatId: string
  selectedGroupId: string
  setSelectedChatId: (id: string) => void
  setSelectedGroupId: (id: string) => void
  
  // Store methods
  get: () => HyperwareChatStore
  set: (partial: HyperwareChatStore | Partial<HyperwareChatStore>) => void
}

const useHyperwareChatStore = create<HyperwareChatStore>()(
  persist(
    (set, get) => ({
      // Direct messages
      hyperware_chats: { "New HyperwareChat": [] },
      addMessage: (msg: NewMessage) => {
        const { hyperware_chats } = get()
        const { hyperware_chat, author, content, timestamp } = msg
        if (!hyperware_chats[hyperware_chat]) {
          hyperware_chats[hyperware_chat] = []
        }
        hyperware_chats[hyperware_chat].push({ author, content, timestamp })
        set({ hyperware_chats })
      },
      
      // Group chats
      groups: [],
      groupMessages: {},
      addGroupMessage: (msg: NewGroupMessage) => {
        const { groupMessages } = get()
        const { group_id, author, content, timestamp } = msg
        if (!groupMessages[group_id]) {
          groupMessages[group_id] = []
        }
        groupMessages[group_id].push({ author, content, timestamp })
        set({ groupMessages })
      },
      addGroup: (group: Group) => {
        const { groups, groupMessages } = get()
        // Check if group already exists
        const existingGroup = groups.find(g => g.id === group.id)
        if (!existingGroup) {
          // Add new group
          const newGroups = [...groups, group]
          // Initialize empty messages array for this group if needed
          if (!groupMessages[group.id]) {
            groupMessages[group.id] = []
          }
          set({ groups: newGroups, groupMessages })
        }
      },
      
      // Contacts
      contacts: [],
      addContact: (contact: Contact) => {
        const { contacts } = get()
        // Check if contact already exists
        const existingContact = contacts.find(c => c.id === contact.id)
        if (!existingContact) {
          set({ contacts: [...contacts, contact] })
        }
      },
      removeContact: (contactId: string) => {
        const { contacts } = get()
        set({ contacts: contacts.filter(c => c.id !== contactId) })
      },
      
      // Selected chat/group state
      selectedChatId: "New HyperwareChat",
      selectedGroupId: "",
      setSelectedChatId: (id: string) => set({ selectedChatId: id, selectedGroupId: "" }),
      setSelectedGroupId: (id: string) => set({ selectedGroupId: id, selectedChatId: "" }),

      get,
      set,
    }),
    {
      name: 'hyperware_chat', // unique name
      storage: createJSONStorage(() => localStorage), // Using localStorage instead of sessionStorage for persistence
    }
  )
)

export default useHyperwareChatStore
