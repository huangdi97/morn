# Morn — Desktop AI Operating System

[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![build](https://img.shields.io/badge/tests-1417_✔_0_✗-brightgreen)](https://github.com/huangdi97/morn)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

> Morn is a desktop AI operating system where AI agents run as processes, Studio is your SDK, and Marketplace is the app store.

---

## Core Features (三台一体)

### 🛠 Workbench
Your daily AI workspace. Chat, computer control, workflow automation, team management, security. Speak to your AI desktop and watch it execute—browse the web, control your mouse and keyboard, run code, and orchestrate multi-agent teams.

### 🎨 Studio
Build and compose AI agents from atomic components: memory, tools, LLMs, channels, personas, skills, and security policies. Drag, drop, and test in real time. Every agent is a composable pipeline.

### 📋 Console
System dashboard. Monitor costs, security events, agent health, system checks, and topology visualization. Governance policies, audit logs, and capacity planning at a glance.

---

## Four User Paths

### End Users
Open Morn, install agents from Marketplace, chat, automate tasks. No code required—just speak your intent.

### Founders
Build AI teams with TeamBuilder. Deploy a virtual COO, CFO, and CTO. Run a one-person company with AI staff that plan, execute, and report.

### Creators
Publish agents and components to Marketplace. Earn revenue from your creations. Reach users across desktop, Telegram, and WeChat.

### Developers
Extend Morn with plugins, MCP tools, custom components, and the WASM sandbox. The entire platform is open source (MIT).

---

## Quick Start

```bash
# Download → Install → Launch → Speak to your AI desktop

git clone https://github.com/huangdi97/morn.git
cd morn-desktop

# Build CLI
cargo build --release --bin morn

# Run (requires an API key)
MORN_API_KEY=sk-xxx cargo run --release -- cli
```

Or download the desktop installer from [Releases](https://github.com/huangdi97/morn/releases).

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                     Desktop UI (Tauri v2)                │
│         Workbench · Studio · Console                    │
├─────────────────────────────────────────────────────────┤
│                     Core Engine (Rust)                   │
│  ┌──────────┐ ┌──────────┐ ┌────────────────────────┐  │
│  │ Event Bus│ │ Model    │ │ 6-Layer Memory         │  │
│  │          │ │ Router   │ │ Working/Episodic/       │  │
│  │          │ │ Cloud/   │ │ Semantic/Procedural/    │  │
│  │          │ │ Local/   │ │ Spatial/Emotional       │  │
│  │          │ │ Hybrid   │ │                         │  │
│  ├──────────┤ ├──────────┤ ├────────────────────────┤  │
│  │ Security │ │ Pipeline │ │ Workflow Engine         │  │
│  │ Guard    │ │ Engine   │ │ DAG · Templates · NL   │  │
│  └──────────┘ └──────────┘ └────────────────────────┘  │
├─────────────────────────────────────────────────────────┤
│  Channels                  │ Computer Control           │
│  Telegram · WeChat ·       │ Browser Ops · Desktop     │
│  Browser Extension ·       │ Ops (mouse/keyboard/      │
│  Desktop Notifications     │ window) · OCR · Screenshot│
│                            │ · VNC Sandbox             │
├─────────────────────────────────────────────────────────┤
│  Plugin System                                          │
│  MCP Bridge · WASM Sandbox · NL-to-Plugin · 5 Examples │
└─────────────────────────────────────────────────────────┘
```

### Core Engine (Rust)
- **Event Bus** — Async pub/sub for inter-module communication
- **Model Router** — Cloud (DeepSeek, OpenAI), local (llama.cpp), or hybrid routing
- **6-Layer Memory** — Working, episodic, semantic, procedural, spatial, emotional
- **Security Guard** — 4-tier constitution: hard block → approval → notification → free
- **Pipeline Engine** — Composable agent pipelines from atomic components
- **Workflow Engine** — DAG-based workflow with NL-to-workflow generation

### Desktop UI (Tauri v2 + React + TypeScript)
- **Workbench** — Chat interface with computer control, team management
- **Studio** — Visual agent builder with drag-and-drop component assembly
- **Console** — Monitoring dashboard with cost, security, and health metrics

### Channels
Telegram, WeChat, browser extension, desktop notifications, REST API, SMTP, CLI.

### Computer Control
Browser operations, desktop operations (mouse/keyboard/window), OCR, screenshot, VNC sandbox for safe remote execution.

### Plugin System
MCP bridge for tool discovery, WASM sandbox for safe third-party code, NL-to-plugin generation, 5 built-in example plugins.

---

## Competitor Comparison

| Feature | Morn | WorkBuddy | Paperclip | OpenHuman | Goose | Dify |
|---------|------|-----------|-----------|-----------|-------|------|
| Desktop native | ✅ Tauri v2 | ❌ Web | ✅ Tauri | ❌ Web | ❌ CLI | ❌ Web |
| AI agent marketplace | ✅ Component + Agent | ❌ | ❌ | ❌ | ❌ | ✅ (template) |
| Offline-first | ✅ Local LLM | ❌ | ❌ | ❌ | ❌ | ❌ |
| Multi-channel | ✅ Telegram/WeChat/Desktop | ❌ | ❌ | ❌ | ❌ | ✅ |
| Self-organization | ✅ TeamBuilder + COO | ❌ | ❌ | ❌ | ❌ | ❌ |
| Privacy-first | ✅ Local + E2EE | ❌ | ❌ | ❌ | ✅ | ❌ |
| Open source | ✅ MIT | ❌ | ✅ | ✅ | ✅ | ✅ |

---

## Tech Stack

| Layer | Technology |
|-------|-----------|
| **Backend** | Rust, Tauri v2, SQLite (rusqlite), tokio async runtime |
| **Frontend** | React 18, TypeScript, Tailwind CSS, DnD Kit |
| **AI** | DeepSeek, OpenAI-compatible, local models (llama.cpp), hybrid routing |
| **Storage** | 6-layer memory (working/episodic/semantic/procedural/spatial/emotional) |

---

## Documentation

- [Setup Guide](docs/setup.md)
- [Architecture](docs/architecture.md)
- [Development](docs/development.md)
- [Studio Guide](docs/studio.md)
- [Console Guide](docs/console.md)
- [Marketplace](docs/market.md)
- [COO Team Management](docs/coo.md)
- [Computer Control](docs/computer-control.md)
- [Channels](docs/channels.md)
- [Components](docs/components.md)
- [Plugin Development](docs/plugin-development.md)
- [Plugin Examples](docs/plugin-examples.md)

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on how to contribute to Morn.

---

## License

MIT License — see [LICENSE](LICENSE) for details.
