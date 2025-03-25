interface WebSocketHandlers {
  onOpen?: () => void;
  onMessage?: (data: any) => void;
  onClose?: () => void;
  onError?: (error: Event) => void;
}

export function setupWebSocket(handlers: WebSocketHandlers): WebSocket {
  // Build WebSocket URL using the base URL from Vite config
  const baseUrl = import.meta.env.BASE_URL;
  const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  const wsUrl = `${wsProtocol}//${window.location.host}${baseUrl}/ws`;
  
  console.log('Connecting to WebSocket URL:', wsUrl);
  const socket = new WebSocket(wsUrl);
  
  socket.onopen = () => {
    console.log('WebSocket connection established');
    if (handlers.onOpen) {
      handlers.onOpen();
    }
  };
  
  socket.onmessage = (event) => {
    try {
      const data = JSON.parse(event.data);
      if (handlers.onMessage) {
        handlers.onMessage(data);
      }
    } catch (error) {
      console.error('Error parsing WebSocket message:', error);
    }
  };
  
  socket.onclose = () => {
    console.log('WebSocket connection closed');
    if (handlers.onClose) {
      handlers.onClose();
    }
  };
  
  socket.onerror = (error) => {
    console.error('WebSocket error:', error);
    if (handlers.onError) {
      handlers.onError(error);
    }
  };
  
  return socket;
}

export function sendWebSocketMessage(socket: WebSocket, type: string, payload: any): void {
  if (socket.readyState === WebSocket.OPEN) {
    const message = {
      type,
      ...payload,
    };
    socket.send(JSON.stringify(message));
  } else {
    console.error('WebSocket is not open');
  }
}