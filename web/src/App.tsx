import { useState, useRef, useEffect } from "react";
import { api } from "./api";
import { ComponentEditor } from "./studio/ComponentEditor";
import { AgentBuilder } from "./studio/AgentBuilder";
import { TestPanel } from "./studio/TestPanel";
import { QuickActions } from "./QuickActions";
import Topology from "./console/Topology";
import SystemInfo from "./console/SystemInfo";
import AdminDashboard from "./console/AdminDashboard";
import CostCenter from "./console/CostCenter";
import Governance from "./console/Governance";
import Security from "./console/Security";
import Marketplace from "./console/Marketplace";
import BotStore from "./store/BotStore";
import { Settings } from "./Settings";
import "./styles/base.css";

type View = "workbench" | "studio" | "console" | "store";

interface Message {
  role: "user" | "assistant";
  content: string;
  timestamp: number;
}

const CHAT_KEY = "morn_chat_history";
const THEME_KEY = "morn_theme";

function App() {
  const [view, setView] = useState<View>("workbench");
  const [messages, setMessages] = useState<Message[]>(() => {
    try {
      const saved = localStorage.getItem(CHAT_KEY);
      return saved ? JSON.parse(saved) : [];
    } catch {
      return [];
    }
  });
  const [input, setInput] = useState("");
  const [status, setStatus] = useState("");
  const [isTyping, setIsTyping] = useState(false);
  const [theme, setTheme] = useState<string>(() => {
    return localStorage.getItem(THEME_KEY) || "dark";
  });
  const [showSettings, setShowSettings] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    localStorage.setItem(CHAT_KEY, JSON.stringify(messages));
  }, [messages]);

  useEffect(() => {
    localStorage.setItem(THEME_KEY, theme);
  }, [theme]);

  useEffect(() => {
    api.getStatus().then((s: any) => {
      setStatus(`v${s.version} | ${s.turn_count} turns`);
    });
  }, []);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, isTyping]);

  const toggleTheme = () => {
    setTheme((prev) => (prev === "dark" ? "light" : "dark"));
  };

  const sendMessage = async () => {
    if (!input.trim()) return;

    const text = input.trim();
    setInput("");

    if (text === "/clear") {
      await api.clearHistory();
      setMessages([]);
      const s: any = await api.getStatus();
      setStatus(`v${s.version} | ${s.turn_count} turns`);
      return;
    }

    const userMsg: Message = { role: "user", content: text, timestamp: Date.now() };
    setMessages((prev) => [...prev, userMsg]);
    setIsTyping(true);

    try {
      const response = await api.sendMessage(text);
      const assistantMsg: Message = { role: "assistant", content: response, timestamp: Date.now() };
      setMessages((prev) => [...prev, assistantMsg]);

      const s: any = await api.getStatus();
      setStatus(`v${s.version} | ${s.turn_count} turns`);
    } catch (e: any) {
      const errorMsg: Message = {
        role: "assistant",
        content: `Error: ${e}`,
        timestamp: Date.now(),
      };
      setMessages((prev) => [...prev, errorMsg]);
    } finally {
      setIsTyping(false);
    }
  };

  const clearHistory = async () => {
    await api.clearHistory();
    setMessages([]);
    const s: any = await api.getStatus();
    setStatus(`v${s.version} | ${s.turn_count} turns`);
  };

  const sendQuickAction = async (text: string) => {
    setInput(text);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  };

  const formatTime = (ts: number) => {
    const d = new Date(ts);
    const hh = String(d.getHours()).padStart(2, "0");
    const mm = String(d.getMinutes()).padStart(2, "0");
    return `${hh}:${mm}`;
  };

  const [studioTab, setStudioTab] = useState<"editor" | "builder" | "test">("builder");
  const [consoleTab, setConsoleTab] = useState<"dashboard" | "topology" | "system" | "cost" | "governance" | "security" | "market">("dashboard");

  const renderStudio = () => (
    <div className="studio-view">
      <nav className="studio-tabs">
        <button className={studioTab === "editor" ? "active" : ""} onClick={() => setStudioTab("editor")}>Component Editor</button>
        <button className={studioTab === "builder" ? "active" : ""} onClick={() => setStudioTab("builder")}>Agent Builder</button>
        <button className={studioTab === "test" ? "active" : ""} onClick={() => setStudioTab("test")}>Test Runner</button>
      </nav>
      <div className="studio-content">
        {studioTab === "editor" && <ComponentEditor />}
        {studioTab === "builder" && <AgentBuilder />}
        {studioTab === "test" && <TestPanel />}
      </div>
    </div>
  );

  const renderConsole = () => (
    <div className="console-view">
      <nav className="console-tabs">
        <button className={consoleTab === "dashboard" ? "active" : ""} onClick={() => setConsoleTab("dashboard")}>Dashboard</button>
        <button className={consoleTab === "topology" ? "active" : ""} onClick={() => setConsoleTab("topology")}>Topology</button>
        <button className={consoleTab === "system" ? "active" : ""} onClick={() => setConsoleTab("system")}>System</button>
        <button className={consoleTab === "cost" ? "active" : ""} onClick={() => setConsoleTab("cost")}>Cost</button>
        <button className={consoleTab === "governance" ? "active" : ""} onClick={() => setConsoleTab("governance")}>Governance</button>
        <button className={consoleTab === "security" ? "active" : ""} onClick={() => setConsoleTab("security")}>Security</button>
        <button className={consoleTab === "market" ? "active" : ""} onClick={() => setConsoleTab("market")}>Marketplace</button>
      </nav>
      <div className="console-content">
        {consoleTab === "dashboard" && <AdminDashboard />}
        {consoleTab === "topology" && <Topology />}
        {consoleTab === "system" && <SystemInfo />}
        {consoleTab === "cost" && <CostCenter />}
        {consoleTab === "governance" && <Governance />}
        {consoleTab === "security" && <Security />}
        {consoleTab === "market" && <Marketplace />}
      </div>
    </div>
  );

  const renderWorkbench = () => (
    <>
      <header className="header">
        <h1>Morn</h1>
        <span className="status">{status}</span>
        <button className="clear-btn" onClick={clearHistory}>
          Clear
        </button>
        <button className="theme-toggle" onClick={toggleTheme}>
          {theme === "dark" ? "\u2600" : "\u263E"}
        </button>
        <button className="settings-btn" onClick={() => setShowSettings(true)}>
          ⚙
        </button>
      </header>

      <main className="chat-area">
        {messages.length === 0 && (
          <div style={{ textAlign: "center", padding: "40px 20px", color: "var(--text-secondary)" }}>
            <div style={{ fontSize: "48px", marginBottom: "12px" }}>🤖</div>
            <h2 style={{ color: "var(--text-primary)", margin: "0 0 8px 0" }}>欢迎使用 Morn</h2>
            <p style={{ fontSize: "14px", margin: "0 0 24px 0" }}>选择快捷任务或直接输入你的问题</p>
            <QuickActions onSend={sendQuickAction} />
          </div>
        )}
        {messages.map((msg, i) => (
          <div key={i} className={`message ${msg.role}`}>
            <div className="avatar">{msg.role === "user" ? "U" : "M"}</div>
            <div className="bubble">
              <div className="bubble-text">{msg.content}</div>
              <span className="timestamp">{formatTime(msg.timestamp)}</span>
            </div>
          </div>
        ))}
        {isTyping && (
          <div className="message assistant">
            <div className="avatar">M</div>
            <div className="bubble typing-indicator">
              <span>typing...</span>
            </div>
          </div>
        )}
        <div ref={messagesEndRef} />
      </main>

      <footer className="input-bar">
        <textarea
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Type a message..."
          rows={1}
        />
        <button onClick={sendMessage} disabled={!input.trim()}>
          Send
        </button>
      </footer>
    </>
  );

  return (
    <div className={`app${theme === "light" ? " light" : ""}`}>
      <nav className="main-tabs">
        <button className={view === "workbench" ? "active" : ""} onClick={() => setView("workbench")}>Workbench</button>
        <button className={view === "studio" ? "active" : ""} onClick={() => setView("studio")}>Studio</button>
        <button className={view === "store" ? "active" : ""} onClick={() => setView("store")}>Store</button>
        <button className={view === "console" ? "active" : ""} onClick={() => setView("console")}>Console</button>
      </nav>
      {view === "workbench" && renderWorkbench()}
      {view === "studio" && renderStudio()}
      {view === "store" && <div className="console-view"><div className="console-content"><BotStore /></div></div>}
      {view === "console" && renderConsole()}
      {showSettings && <Settings onClose={() => setShowSettings(false)} />}
    </div>
  );
}

export default App;
