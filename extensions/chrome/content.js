// Morn AI Assistant - Content Script
// Extracts page information, listens for text selection events,
// and displays toast notifications from Morn.

// --- Toast Notification System ---

function createToastContainer() {
  const container = document.createElement('div');
  container.id = 'morn-toast-container';
  container.style.cssText = `
    position: fixed;
    top: 20px;
    right: 20px;
    z-index: 2147483647;
    display: flex;
    flex-direction: column;
    gap: 8px;
    pointer-events: none;
  `;
  document.body.appendChild(container);
  return container;
}

function showToast(message, type = 'info', duration = 4000) {
  let container = document.getElementById('morn-toast-container');
  if (!container) {
    container = createToastContainer();
  }

  const toast = document.createElement('div');
  toast.className = `morn-toast morn-toast-${type}`;
  toast.style.cssText = `
    background: ${type === 'error' ? '#e74c3c' : type === 'success' ? '#2ecc71' : '#3498db'};
    color: white;
    padding: 12px 20px;
    border-radius: 8px;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    font-size: 14px;
    line-height: 1.4;
    box-shadow: 0 4px 12px rgba(0,0,0,0.15);
    opacity: 0;
    transform: translateX(50px);
    transition: opacity 0.3s ease, transform 0.3s ease;
    max-width: 400px;
    word-wrap: break-word;
    pointer-events: auto;
  `;
  toast.textContent = typeof message === 'string' ? message : JSON.stringify(message);

  container.appendChild(toast);

  // Animate in
  requestAnimationFrame(() => {
    toast.style.opacity = '1';
    toast.style.transform = 'translateX(0)';
  });

  // Auto dismiss
  const dismissTimer = setTimeout(() => {
    dismissToast(toast);
  }, duration);

  // Dismiss on click
  toast.addEventListener('click', () => {
    clearTimeout(dismissTimer);
    dismissToast(toast);
  });
}

function dismissToast(toast) {
  toast.style.opacity = '0';
  toast.style.transform = 'translateX(50px)';
  setTimeout(() => {
    if (toast.parentNode) {
      toast.parentNode.removeChild(toast);
    }
  }, 300);
}

// --- Text Selection Detection ---

let selectionTimeout = null;
let lastSelectedText = '';

document.addEventListener('mouseup', () => {
  // Debounce selection events
  if (selectionTimeout) {
    clearTimeout(selectionTimeout);
  }

  selectionTimeout = setTimeout(() => {
    const selection = window.getSelection();
    const selectedText = selection ? selection.toString().trim() : '';

    if (selectedText && selectedText !== lastSelectedText) {
      lastSelectedText = selectedText;
      sendSelectionToBackground(selectedText);
    }
  }, 300);
});

// Also capture selection via keyboard (Shift+Arrow, Ctrl+A, etc.)
document.addEventListener('keyup', (event) => {
  // Only respond to text-selection-related keys
  if (event.shiftKey || event.ctrlKey || event.metaKey) {
    if (selectionTimeout) {
      clearTimeout(selectionTimeout);
    }

    selectionTimeout = setTimeout(() => {
      const selection = window.getSelection();
      const selectedText = selection ? selection.toString().trim() : '';

      if (selectedText && selectedText !== lastSelectedText) {
        lastSelectedText = selectedText;
        sendSelectionToBackground(selectedText);
      }
    }, 300);
  }
});

function sendSelectionToBackground(text) {
  chrome.runtime.sendMessage({
    source: 'content',
    payload: {
      type: 'textSelection',
      text: text,
      title: document.title,
      url: window.location.href
    }
  }).catch(() => {
    // Background may not be available yet
  });
}

// --- Page Information Extraction ---

function getPageInfo() {
  return {
    title: document.title,
    url: window.location.href,
    meta: getMetaTags()
  };
}

function getMetaTags() {
  const metas = document.querySelectorAll('meta');
  const metaObj = {};
  metas.forEach((meta) => {
    const name = meta.getAttribute('name') || meta.getAttribute('property');
    const content = meta.getAttribute('content');
    if (name && content) {
      metaObj[name] = content;
    }
  });
  return metaObj;
}

// Send page info on load
function sendPageInfo() {
  // Delay slightly to ensure DOM is ready
  setTimeout(() => {
    chrome.runtime.sendMessage({
      source: 'content',
      payload: {
        type: 'pageInfo',
        ...getPageInfo()
      }
    }).catch(() => {
      // Background may not be ready yet
    });
  }, 500);
}

// --- Message Listener from Background ---

chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (!message || message.source !== 'morn') return;

  const data = message.data;

  // If data is a string, try to parse as JSON
  let payload = data;
  if (typeof data === 'string') {
    try {
      payload = JSON.parse(data);
    } catch (e) {
      // Not JSON, use as-is
    }
  }

  // Handle different message types
  if (payload.type === 'notification') {
    showToast(payload.message, payload.level || 'info', payload.duration || 4000);
  } else if (payload.type === 'toast') {
    showToast(payload.text || payload.message, payload.level || 'info', payload.duration || 4000);
  } else {
    // Default: show as toast
    const displayText = typeof payload === 'string' ? payload :
      payload.text || payload.message || JSON.stringify(payload);
    showToast(displayText, 'info', 4000);
  }

  sendResponse({ received: true });
  return true;
});

// --- Initialization ---

// Send page info when the page loads
if (document.readyState === 'complete') {
  sendPageInfo();
} else {
  window.addEventListener('load', sendPageInfo);
}

console.log('[Morn] Content script loaded on:', window.location.href);
