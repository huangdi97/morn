const messages = document.getElementById('messages');
const input = document.getElementById('chat-input');
const sendButton = document.getElementById('send-btn');

let currentContext = null;
let contextBadge = null;
let history = [];

async function getActiveTab() {
  const tabs = await chrome.tabs.query({ active: true, currentWindow: true });
  return tabs[0] || null;
}

async function getPageContext() {
  const tab = await getActiveTab();
  if (!tab) {
    return null;
  }

  try {
    return await chrome.tabs.sendMessage(tab.id, { action: 'morn:getPageContext' });
  } catch (error) {
    return {
      url: tab.url || '',
      title: tab.title || '',
      content: '',
      selection: null,
    };
  }
}

function renderContextBadge(context) {
  if (!contextBadge) {
    contextBadge = document.createElement('div');
    contextBadge.className = 'context-badge';
    messages.prepend(contextBadge);
  }

  contextBadge.textContent = context && context.title
    ? `Context: ${context.title}`
    : 'Context: current page';
}

function addMessage(role, text) {
  const node = document.createElement('div');
  node.className = `msg ${role}`;
  node.textContent = text;
  messages.appendChild(node);
  messages.scrollTop = messages.scrollHeight;
  history.push({ role, text });
  return node;
}

function buildContextPrompt(query) {
  const context = currentContext || {};
  const recentHistory = history
    .slice(-8)
    .map((item) => `${item.role}: ${item.text}`)
    .join('\n');

  return [
    query,
    '',
    'Recent conversation:',
    recentHistory,
    '',
    'Page context:',
    `URL: ${context.url || ''}`,
    `Title: ${context.title || ''}`,
    `Selection: ${context.selection || ''}`,
    `Content: ${context.content || ''}`,
  ].join('\n');
}

async function sendToMorn(payload) {
  return chrome.runtime.sendMessage({ action: 'morn:send', payload });
}

async function refreshContext() {
  currentContext = await getPageContext();
  renderContextBadge(currentContext);

  if (currentContext) {
    await sendToMorn({ type: 'page_context', ...currentContext });
  }
}

async function sendChat() {
  const text = input.value.trim();
  if (!text) {
    return;
  }

  input.value = '';
  sendButton.disabled = true;

  addMessage('user', text);
  const assistantNode = addMessage('assistant', '...');

  try {
    currentContext = await getPageContext();
    renderContextBadge(currentContext);

    const response = await sendToMorn({
      type: 'chat',
      text: buildContextPrompt(text),
    });

    assistantNode.textContent = response.type === 'reply'
      ? response.text
      : response.message || 'No response';
    history[history.length - 1].text = assistantNode.textContent;
  } catch (error) {
    assistantNode.textContent = error.message;
    history[history.length - 1].text = error.message;
  } finally {
    sendButton.disabled = false;
    input.focus();
  }
}

sendButton.addEventListener('click', sendChat);
input.addEventListener('keydown', (event) => {
  if (event.key === 'Enter' && !event.shiftKey) {
    event.preventDefault();
    sendChat();
  }
});

chrome.runtime.onMessage.addListener((message) => {
  if (message && message.action === 'morn:wsMessage' && message.payload) {
    if (message.payload.type === 'suggestion') {
      addMessage('assistant', message.payload.text);
    }
  }
});

refreshContext().catch((error) => {
  addMessage('assistant', error.message);
});
input.focus();
