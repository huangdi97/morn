// Morn AI Assistant - Background Service Worker
// Maintains WebSocket connection to Morn server, forwards messages
// between content scripts and the Morn WebSocket server.

const DEFAULT_WS_PORT = 9876;
let ws = null;
let reconnectTimer = null;
let reconnectAttempt = 0;
const MAX_RECONNECT_DELAY = 30000; // 30 seconds max backoff
const BASE_RECONNECT_DELAY = 1000; // 1 second initial
const PING_INTERVAL = 30000; // 30 seconds

let pingInterval = null;
let isConnected = false;

// --- Configuration ---

function getWsUrl() {
  return new Promise((resolve) => {
    chrome.storage.local.get(['wsPort'], (result) => {
      const port = result.wsPort || DEFAULT_WS_PORT;
      resolve(`ws://localhost:${port}`);
    });
  });
}

// --- WebSocket Connection ---

async function connect() {
  // Clean up any existing connection
  disconnect();

  const url = await getWsUrl();
  console.log(`[Morn] Connecting to ${url}...`);

  try {
    ws = new WebSocket(url);

    ws.onopen = () => {
      console.log(`[Morn] Connected to ${url}`);
      isConnected = true;
      reconnectAttempt = 0;
      startPing();
      broadcastStatus(true);
    };

    ws.onmessage = (event) => {
      console.log('[Morn] Received message from server:', event.data);
      // Forward to all active tabs
      forwardToContentScripts(event.data);
    };

    ws.onerror = (error) => {
      console.error('[Morn] WebSocket error:', error);
      // onclose will handle reconnection
    };

    ws.onclose = (event) => {
      console.log(`[Morn] Disconnected (code: ${event.code})`);
      isConnected = false;
      stopPing();
      broadcastStatus(false);
      scheduleReconnect();
    };
  } catch (error) {
    console.error('[Morn] Failed to create WebSocket:', error);
    isConnected = false;
    broadcastStatus(false);
    scheduleReconnect();
  }
}

function disconnect() {
  if (pingInterval) {
    clearInterval(pingInterval);
    pingInterval = null;
  }
  if (reconnectTimer) {
    clearTimeout(reconnectTimer);
    reconnectTimer = null;
  }
  if (ws) {
    ws.onopen = null;
    ws.onmessage = null;
    ws.onerror = null;
    ws.onclose = null;
    if (ws.readyState === WebSocket.OPEN || ws.readyState === WebSocket.CONNECTING) {
      ws.close();
    }
    ws = null;
  }
  isConnected = false;
}

function scheduleReconnect() {
  if (reconnectTimer) {
    clearTimeout(reconnectTimer);
  }

  // Exponential backoff with jitter
  const delay = Math.min(
    BASE_RECONNECT_DELAY * Math.pow(2, reconnectAttempt),
    MAX_RECONNECT_DELAY
  ) + Math.random() * 1000;

  reconnectAttempt++;
  console.log(`[Morn] Reconnecting in ${Math.round(delay)}ms (attempt ${reconnectAttempt})`);

  reconnectTimer = setTimeout(() => {
    connect();
  }, delay);
}

// --- Ping/Pong Keep Alive ---

function startPing() {
  stopPing();
  pingInterval = setInterval(() => {
    if (ws && ws.readyState === WebSocket.OPEN) {
      try {
        ws.send(JSON.stringify({ type: 'ping' }));
        console.log('[Morn] Sent ping');
      } catch (error) {
        console.error('[Morn] Ping failed:', error);
      }
    }
  }, PING_INTERVAL);
}

function stopPing() {
  if (pingInterval) {
    clearInterval(pingInterval);
    pingInterval = null;
  }
}

// --- Message Forwarding ---

function forwardToContentScripts(data) {
  chrome.tabs.query({}, (tabs) => {
    for (const tab of tabs) {
      chrome.tabs.sendMessage(tab.id, {
        source: 'morn',
        data: data
      }).catch(() => {
        // Tab may not have content script loaded, ignore
      });
    }
  });
}

// Listen for messages from content scripts and forward to WebSocket
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (!message || !message.source) return;

  if (message.source === 'content') {
    // Forward to WebSocket if connected
    if (ws && ws.readyState === WebSocket.OPEN) {
      const payload = {
        ...message.payload,
        tabId: sender.tab?.id,
        url: sender.tab?.url
      };
      ws.send(JSON.stringify(payload));
      sendResponse({ status: 'sent' });
    } else {
      sendResponse({ status: 'disconnected' });
    }
    return true; // Keep channel open for async response
  }

  if (message.source === 'popup') {
    if (message.action === 'getStatus') {
      sendResponse({
        connected: isConnected,
        reconnectAttempt: reconnectAttempt
      });
    }
    if (message.action === 'reconnect') {
      reconnectAttempt = 0;
      connect();
      sendResponse({ status: 'reconnecting' });
    }
    return true;
  }
});

// --- Status Broadcasting ---

function broadcastStatus(connected) {
  chrome.runtime.sendMessage({
    source: 'background',
    type: 'connectionStatus',
    connected: connected
  }).catch(() => {
    // No listeners yet, ignore
  });
}

// --- Extension Lifecycle ---

chrome.runtime.onInstalled.addListener(() => {
  console.log('[Morn] Extension installed. Starting connection...');
  connect();
});

chrome.runtime.onStartup.addListener(() => {
  console.log('[Morn] Extension started. Connecting...');
  connect();
});

// Start connecting immediately
connect();
