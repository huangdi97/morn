import { useState, useRef, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ComponentEditor } from "./studio/ComponentEditor";
import { AgentBuilder } from "./studio/AgentBuilder";
import { TestPanel } from "./studio/TestPanel";
import Topology from "./console/Topology";
import SystemInfo from "./console/SystemInfo";
import Dashboard from "./console/Dashboard";
import CostCenter from "./console/CostCenter";
import Governance from "./console/Governance";
import Security from "./console/Security";
import Marketplace from "./console/Marketplace";

type View = "workbench" | "studio" | "console";

interface Message {
  role: "user" | "assistant";
  content: string;
}

function App() {
  const [view, setView] = useState<View>("workbench");
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState("");
  const [status, setStatus] = useState("");
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    invoke("get_status").then((s: any) => {
      setStatus(`v${s.version} | ${s.turn_count} turns`);
    });
  }, []);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  const sendMessage = async () => {
    if (!input.trim()) return;

    const userMsg: Message = { role: "user", content: input };
    setMessages((prev) => [...prev, userMsg]);
    setInput("");

    try {
      const response = await invoke<string>("send_message", { text: input });
      const assistantMsg: Message = { role: "assistant", content: response };
      setMessages((prev) => [...prev, assistantMsg]);

      const s: any = await invoke("get_status");
      setStatus(`v${s.version} | ${s.turn_count} turns`);
    } catch (e: any) {
      const errorMsg: Message = {
        role: "assistant",
        content: `Error: ${e}`,
      };
      setMessages((prev) => [...prev, errorMsg]);
    }
  };

  const clearHistory = async () => {
    await invoke("clear_history");
    setMessages([]);
    const s: any = await invoke("get_status");
    setStatus(`v${s.version} | ${s.turn_count} turns`);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
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
        {consoleTab === "dashboard" && <Dashboard />}
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
      </header>

      <main className="chat-area">
        {messages.map((msg, i) => (
          <div key={i} className={`message ${msg.role}`}>
            <div className="avatar">{msg.role === "user" ? "U" : "M"}</div>
            <div className="bubble">{msg.content}</div>
          </div>
        ))}
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
    <div className="app">
      <nav className="main-tabs">
        <button className={view === "workbench" ? "active" : ""} onClick={() => setView("workbench")}>Workbench</button>
        <button className={view === "studio" ? "active" : ""} onClick={() => setView("studio")}>Studio</button>
        <button className={view === "console" ? "active" : ""} onClick={() => setView("console")}>Console</button>
      </nav>
      {view === "workbench" && renderWorkbench()}
      {view === "studio" && renderStudio()}
      {view === "console" && renderConsole()}
    </div>
  );
}

export default App;
