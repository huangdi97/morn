// Morn AI Assistant - Popup Script
// Handles UI updates, connection status, and user interactions.

document.addEventListener('DOMContentLoaded', () => {
  // DOM Elements
  const statusDot = document.getElementById('statusDot');
  const statusText = document.getElementById('statusText');
  const detailedStatus = document.getElementById('detailedStatus');
  const serverAddress = document.getElementById('serverAddress');
  const pageTitle = document.getElementById('pageTitle');
  const pageUrl = document.getElementById('pageUrl');
  const toggleMorn = document.getElementById('toggleMorn');
  const reconnectBtn = document.getElementById('reconnectBtn');

  let connected = false;
  let mornEnabled = true;

  // --- Initialize ---

  // Load saved toggle state
  chrome.storage.local.get(['mornEnabled'], (result) => {
    mornEnabled = result.mornEnabled !== false; // default true
    toggleMorn.checked = mornEnabled;
  });

  // Get current tab info
  chrome.tabs.query({ active: true, currentWindow: true }, (tabs) => {
    if (tabs && tabs.length > 0) {
      const tab = tabs[0];
      pageTitle.textContent = tab.title || '(no title)';
      pageUrl.textContent = tab.url || '(no URL)';
    }
  });

  // Get server address
  chrome.storage.local.get(['wsPort'], (result) => {
    const port = result.wsPort || 9876;
    serverAddress.textContent = `ws://localhost:${port}`;
  });

  // --- Connection Status ---

  function updateStatus(isConnected) {
    connected = isConnected;
    statusDot.className = 'status-dot';
    if (isConnected) {
      statusDot.classList.add('connected');
      statusText.textContent = 'Connected';
      detailedStatus.textContent = 'Connected';
    } else {
      statusDot.classList.add('disconnected');
      statusText.textContent = 'Disconnected';
      detailedStatus.textContent = 'Disconnected';
    }
  }

  // Initial status
  updateStatus(false);

  // Listen for status updates from background
  chrome.runtime.onMessage.addListener((message) => {
    if (message && message.source === 'background' && message.type === 'connectionStatus') {
      updateStatus(message.connected);
    }
  });

  // Query current status from background
  chrome.runtime.sendMessage({ source: 'popup', action: 'getStatus' }, (response) => {
    if (response) {
      updateStatus(response.connected);
      if (response.reconnectAttempt > 0) {
        statusText.textContent = 'Reconnecting...';
        statusDot.className = 'status-dot connecting';
        detailedStatus.textContent = `Reconnecting (attempt ${response.reconnectAttempt})`;
      }
    }
  });

  // --- Toggle Handler ---

  toggleMorn.addEventListener('change', () => {
    mornEnabled = toggleMorn.checked;
    chrome.storage.local.set({ mornEnabled: mornEnabled });

    // Send toggle state to background
    chrome.runtime.sendMessage({
      source: 'popup',
      action: 'toggleMorn',
      enabled: mornEnabled
    });

    // Toggle content script behavior
    chrome.tabs.query({ active: true, currentWindow: true }, (tabs) => {
      if (tabs && tabs.length > 0) {
        chrome.tabs.sendMessage(tabs[0].id, {
          source: 'popup',
          action: 'toggleMorn',
          enabled: mornEnabled
        }).catch(() => {
          // Content script may not be loaded
        });
      }
    });
  });

  // --- Reconnect Button ---

  reconnectBtn.addEventListener('click', () => {
    statusText.textContent = 'Reconnecting...';
    statusDot.className = 'status-dot connecting';
    detailedStatus.textContent = 'Reconnecting...';
    reconnectBtn.disabled = true;
    reconnectBtn.textContent = 'Connecting...';

    chrome.runtime.sendMessage({ source: 'popup', action: 'reconnect' }, (response) => {
      if (response && response.status === 'reconnecting') {
        // Status will update via onMessage listener when connected
      }
    });

    // Re-enable button after timeout
    setTimeout(() => {
      reconnectBtn.disabled = false;
      reconnectBtn.textContent = 'Reconnect';
    }, 5000);
  });
});
