# Plugin Examples

Five minimal plugin examples demonstrating each plugin type in Morn.

## 1. plugin-theme-simple

**Type:** Theme

A minimal theme plugin that customises UI colours via CSS custom properties.

**Files:**
- `manifest.json` — Plugin metadata
- `theme.css` — CSS custom properties for theming
- `main.js` — Entry point (logs load)

## 2. plugin-channel-echo

**Type:** Channel

An echo channel plugin that mirrors messages back to the sender.

**Files:**
- `manifest.json` — Plugin metadata
- `main.js` — EchoChannel class with connect/send/onMessage/disconnect

## 3. plugin-tool-greeter

**Type:** Tool

A tool plugin that greets users by name. Demonstrates the minimal tool interface.

**Files:**
- `manifest.json` — Plugin metadata
- `main.js` — GreeterTool class with async run(args)

## 4. plugin-knowledge-faq

**Type:** Knowledge

An FAQ knowledge base plugin that serves question-answer pairs.

**Files:**
- `manifest.json` — Plugin metadata
- `data.json` — Structured FAQ entries
- `main.js` — FaqKnowledge class with query method

## 5. plugin-protocol-echo

**Type:** Protocol

An echo protocol plugin for testing MCP-style communication.

**Files:**
- `manifest.json` — Plugin metadata
- `protocol.json` — Endpoint definitions
- `main.js` — EchoProtocol class with handleRequest

## How to Use

1. Copy any example directory to your Morn plugins folder
2. Run `PluginManager::scan()` or restart Morn
3. The plugin will be discovered and available

Source: `src/themes/examples/`