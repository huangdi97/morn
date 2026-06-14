import { useState, useRef, useEffect } from "react";
import { api } from "./api";
import { ComponentEditor } from "./studio/ComponentEditor";
import { AgentBuilder } from "./studio/AgentBuilder";
import { TeamBuilder } from "./studio/TeamBuilder";
import { TestPanel } from "./studio/TestPanel";
import { TeamTemplateSelector } from "./studio/TeamTemplateSelector";
import { DevZone } from "./studio/DevZone";
import { ComponentTypeManager } from "./studio/ComponentTypeManager";
import { McpManager } from "./studio/McpManager";
import { QuickActions } from "./QuickActions";
import Topology from "./console/Topology";
import SystemInfo from "./console/SystemInfo";
import AdminDashboard from "./console/AdminDashboard";
import CostCenter from "./console/CostCenter";
import RoiCalculator from "./console/RoiCalculator";
import SystemCheck from "./console/SystemCheck";
import Governance from "./console/Governance";
import Security from "./console/Security";
import Marketplace from "./console/Marketplace";
import NotificationManager from "./console/NotificationManager";
import MemoryManager from "./console/MemoryManager";
import Connections from "./console/Connections";
import UserJourney from "./console/UserJourney";
import AudioPanel from "./console/AudioPanel";
import CostPanel from "./console/CostPanel";
import LocalModelPanel from "./console/LocalModelPanel";
import AnalyticsPanel from "./console/AnalyticsPanel";
import SandboxPanel from "./console/SandboxPanel";
import ProactivePanel from "./console/ProactivePanel";
import BusinessTemplates from "./console/BusinessTemplates";
import BotStore from "./store/BotStore";
import { Settings } from "./Settings";
import StatusBar from "./StatusBar";
import "./styles/base.css";
import "./styles/skeleton.css";
import "./styles/dashboard.css";

type View = "workbench" | "studio" | "console" | "store";

interface Message {
  role: "user" | "assistant";
  content: string;
  timestamp: number;
}

const CHAT_KEY = "morn_chat_history";
const THEME_KEY = "morn-theme";

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
    return localStorage.getItem(THEME_KEY) || "cyber";
  });
  const [showSettings, setShowSettings] = useState(false);
  const [sendingIndex, setSendingIndex] = useState<number | null>(null);
  const [loading, setLoading] = useState<Record<string, boolean>>({ workbench: true, studio: true, console: true });
  const [workStep, setWorkStep] = useState(0);
  const [workVisible, setWorkVisible] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    localStorage.setItem(CHAT_KEY, JSON.stringify(messages));
  }, [messages]);

  useEffect(() => {
    localStorage.setItem(THEME_KEY, theme);
  }, [theme]);

  useEffect(() => {
    const handler = (e: StorageEvent) => {
      if (e.key === 'morn-theme') setTheme(e.newValue || 'cyber');
    };
    window.addEventListener('storage', handler);
    return () => window.removeEventListener('storage', handler);
  }, []);

  useEffect(() => {
    api.getStatus().then((s: any) => {
      setStatus(`v${s.version} | ${s.turn_count} turns`);
    });
  }, []);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, isTyping]);

  useEffect(() => {
    if (!isTyping) return;
    setWorkStep(0);
    setWorkVisible(true);
  }, [isTyping]);

  useEffect(() => {
    if (!workVisible) return;
    if (workStep >= 4) {
      const t = setTimeout(() => setWorkVisible(false), 2000);
      return () => clearTimeout(t);
    }
    const t = setTimeout(() => setWorkStep(s => s + 1), 1000);
    return () => clearTimeout(t);
  }, [workVisible, workStep]);

  useEffect(() => {
    if (view !== "store") {
      setLoading(prev => ({ ...prev, [view]: true }));
      const t = setTimeout(() => setLoading(prev => ({ ...prev, [view]: false })), 500);
      return () => clearTimeout(t);
    }
  }, [view]);

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
    setMessages((prev) => {
      const next = [...prev, userMsg];
      setSendingIndex(next.length - 1);
      return next;
    });
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
      setTimeout(() => setSendingIndex(null), 500);
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

  const [studioTab, setStudioTab] = useState<"editor" | "builder" | "test" | "teams" | "team" | "dev" | "types" | "mcp">("builder");
  const [consoleTab, setConsoleTab] = useState<"dashboard" | "journey" | "topology" | "system" | "cost" | "roi" | "governance" | "security" | "market" | "system_check" | "notifications" | "memory" | "connections" | "audio" | "cost_tracking" | "local_models" | "analytics" | "sandbox" | "proactive" | "business">("dashboard");

  const SkeletonChat = () => (
    <div className="skeleton-chat">
      {[1, 2, 3, 4].map(i => (
        <div key={i} className={`skeleton-chat-row ${i % 2 === 0 ? "user" : ""}`}>
          <div className="skeleton skeleton-avatar" />
          <div className="skeleton skeleton-bubble" />
        </div>
      ))}
    </div>
  );

  const SkeletonStudio = () => (
    <div className="skeleton-studio">
      <div className="skeleton-studio-nav">
        {[1, 2, 3].map(i => <div key={i} className="skeleton skeleton-studio-nav-item" />)}
      </div>
      <div className="skeleton-studio-body">
        <div className="skeleton-studio-sidebar">
          {[1, 2, 3, 4, 5].map(i => <div key={i} className="skeleton skeleton-studio-sidebar-item" />)}
        </div>
        <div className="skeleton-studio-main">
          <div className="skeleton skeleton-studio-main-item half" />
          <div className="skeleton skeleton-studio-main-item" />
          <div className="skeleton skeleton-studio-main-item" />
          <div className="skeleton skeleton-studio-main-item tall" />
          <div className="skeleton skeleton-studio-main-item half" />
        </div>
      </div>
    </div>
  );

  const SkeletonConsole = () => (
    <div className="skeleton-console">
      <div className="skeleton-console-grid">
        {[1, 2, 3, 4, 5, 6].map(i => <div key={i} className="skeleton skeleton-console-card" />)}
      </div>
      <div className="skeleton skeleton-console-wide" />
      <div className="skeleton-console-charts">
        <div className="skeleton skeleton-console-chart" />
        <div className="skeleton skeleton-console-chart" />
      </div>
    </div>
  );

  const AGENTS = [
    { name: "Planner", active: true },
    { name: "Coder", active: true },
    { name: "Reviewer", active: true },
    { name: "Tester", active: true },
    { name: "Monitor", active: true },
    { name: "Deployer", active: false },
    { name: "Optimizer", active: false },
    { name: "Analyst", active: false },
  ];

  const WORK_STEPS = ["Thinking...", "Planning...", "Working...", "Done ✓"];

  function AgentBar() {
    const maxVisible = 6;
    const visible = AGENTS.slice(0, maxVisible);
    const extra = AGENTS.length - maxVisible;

    return (
      <div className="agent-bar">
        {visible.map(agent => (
          <span key={agent.name} className="agent-item">
            <span className={`agent-dot ${agent.active ? "active" : "inactive"}`} />
            {agent.name}
          </span>
        ))}
        {extra > 0 && <span className="agent-extra">+{extra} more</span>}
      </div>
    );
  }

  function WorkLog({ visible, step }: { visible: boolean; step: number }) {
    if (!visible) return null;
    const label = step < WORK_STEPS.length ? WORK_STEPS[step] : WORK_STEPS[WORK_STEPS.length - 1];

    return (
      <div className="work-log">
        <span className="work-log-label">{label}</span>
        <div className="work-log-dots">
          {[0, 1, 2, 3].map(i => (
            <span key={i} className={`work-dot ${i <= step ? "filled" : ""}`} />
          ))}
        </div>
      </div>
    );
  }

  const renderStudio = () => (
    <div className="studio-view">
      <nav className="studio-tabs">
        <button className={studioTab === "editor" ? "active" : ""} onClick={() => setStudioTab("editor")}>Component Editor</button>
        <button className={studioTab === "builder" ? "active" : ""} onClick={() => setStudioTab("builder")}>Agent Builder</button>
        <button className={studioTab === "teams" ? "active" : ""} onClick={() => setStudioTab("teams")}>Teams</button>
        <button className={studioTab === "team" ? "active" : ""} onClick={() => setStudioTab("team")}>Team Builder</button>
        <button className={studioTab === "dev" ? "active" : ""} onClick={() => setStudioTab("dev")}>Dev</button>
        <button className={studioTab === "types" ? "active" : ""} onClick={() => setStudioTab("types")}>Types</button>
        <button className={studioTab === "mcp" ? "active" : ""} onClick={() => setStudioTab("mcp")}>MCP</button>
        <button className={studioTab === "test" ? "active" : ""} onClick={() => setStudioTab("test")}>Test Runner</button>
      </nav>
      <div className="studio-content">
        {loading.studio ? <SkeletonStudio /> : (
          <>
            {studioTab === "editor" && <ComponentEditor />}
            {studioTab === "builder" && <AgentBuilder />}
            {studioTab === "teams" && (
              <TeamTemplateSelector
                onSelect={async (template) => {
                  try {
                    await api.createTeam(template.name, template.description, "default-user");
                    alert(`团队 "${template.name}" 创建成功`);
                  } catch (e: any) {
                    alert(`创建失败: ${e}`);
                  }
                }}
              />
            )}
            {studioTab === "team" && <TeamBuilder />}
            {studioTab === "test" && <TestPanel />}
            {studioTab === "dev" && <DevZone />}
            {studioTab === "types" && <ComponentTypeManager />}
            {studioTab === "mcp" && <McpManager />}
          </>
        )}
      </div>
    </div>
  );

  const renderConsole = () => (
    <div className="console-view">
      <nav className="console-tabs">
        <button className={consoleTab === "dashboard" ? "active" : ""} onClick={() => setConsoleTab("dashboard")}>Dashboard</button>
        <button className={consoleTab === "journey" ? "active" : ""} onClick={() => setConsoleTab("journey")}>Journey</button>
        <button className={consoleTab === "topology" ? "active" : ""} onClick={() => setConsoleTab("topology")}>Topology</button>
        <button className={consoleTab === "system" ? "active" : ""} onClick={() => setConsoleTab("system")}>System</button>
        <button className={consoleTab === "cost" ? "active" : ""} onClick={() => setConsoleTab("cost")}>Cost</button>
        <button className={consoleTab === "roi" ? "active" : ""} onClick={() => setConsoleTab("roi")}>ROI</button>
        <button className={consoleTab === "governance" ? "active" : ""} onClick={() => setConsoleTab("governance")}>Governance</button>
        <button className={consoleTab === "security" ? "active" : ""} onClick={() => setConsoleTab("security")}>Security</button>
        <button className={consoleTab === "market" ? "active" : ""} onClick={() => setConsoleTab("market")}>Marketplace</button>
        <button className={consoleTab === "system_check" ? "active" : ""} onClick={() => setConsoleTab("system_check")}>Self-Check</button>
        <button className={consoleTab === "notifications" ? "active" : ""} onClick={() => setConsoleTab("notifications")}>Notifications</button>
        <button className={consoleTab === "memory" ? "active" : ""} onClick={() => setConsoleTab("memory")}>Memory</button>
        <button className={consoleTab === "connections" ? "active" : ""} onClick={() => setConsoleTab("connections")}>Connections</button>
        <button className={consoleTab === "audio" ? "active" : ""} onClick={() => setConsoleTab("audio")}>Audio</button>
        <button className={consoleTab === "cost_tracking" ? "active" : ""} onClick={() => setConsoleTab("cost_tracking")}>Cost</button>
        <button className={consoleTab === "local_models" ? "active" : ""} onClick={() => setConsoleTab("local_models")}>Local Models</button>
        <button className={consoleTab === "analytics" ? "active" : ""} onClick={() => setConsoleTab("analytics")}>Analytics</button>
        <button className={consoleTab === "sandbox" ? "active" : ""} onClick={() => setConsoleTab("sandbox")}>Sandbox</button>
        <button className={consoleTab === "proactive" ? "active" : ""} onClick={() => setConsoleTab("proactive")}>Proactive</button>
        <button className={consoleTab === "business" ? "active" : ""} onClick={() => setConsoleTab("business")}>Business</button>
      </nav>
      <div className="console-content">
        {loading.console ? <SkeletonConsole /> : (
          <>
            {consoleTab === "dashboard" && <AdminDashboard />}
            {consoleTab === "journey" && <UserJourney />}
            {consoleTab === "topology" && <Topology />}
            {consoleTab === "system" && <SystemInfo />}
            {consoleTab === "cost" && <CostCenter />}
            {consoleTab === "roi" && <RoiCalculator />}
            {consoleTab === "governance" && <Governance />}
            {consoleTab === "security" && <Security />}
            {consoleTab === "market" && <Marketplace />}
            {consoleTab === "system_check" && <SystemCheck />}
            {consoleTab === "notifications" && <NotificationManager />}
            {consoleTab === "memory" && <MemoryManager />}
            {consoleTab === "connections" && <Connections />}
            {consoleTab === "audio" && <AudioPanel />}
            {consoleTab === "cost_tracking" && <CostPanel />}
            {consoleTab === "local_models" && <LocalModelPanel />}
            {consoleTab === "analytics" && <AnalyticsPanel />}
            {consoleTab === "sandbox" && <SandboxPanel />}
            {consoleTab === "proactive" && <ProactivePanel />}
            {consoleTab === "business" && <BusinessTemplates />}
          </>
        )}
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
        <button className="settings-btn" onClick={() => setShowSettings(true)}>
          ⚙
        </button>
      </header>

      <AgentBar />
      <WorkLog visible={workVisible} step={workStep} />

      <main className="chat-area">
        {loading.workbench && messages.length === 0 ? <SkeletonChat /> : (
          <>
        {messages.length === 0 && (
          <div style={{ textAlign: "center", padding: "40px 20px", color: "var(--text-secondary)" }}>
            <div style={{ fontSize: "48px", marginBottom: "12px" }}>🤖</div>
            <h2 style={{ color: "var(--text-primary)", margin: "0 0 8px 0" }}>欢迎使用 Morn</h2>
            <p style={{ fontSize: "14px", margin: "0 0 24px 0" }}>选择快捷任务或直接输入你的问题</p>
            <QuickActions onSend={sendQuickAction} />
          </div>
        )}
        {messages.map((msg, i) => (
          <div key={i} className={`message ${msg.role}${i === sendingIndex ? ' sending' : ''}`}>
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
              <span></span><span></span><span></span>
            </div>
          </div>
        )}
        <div ref={messagesEndRef} />
          </>
        )}
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
    <div className="app" data-theme={theme}>
      <nav className="main-tabs">
        <button className={view === "workbench" ? "active" : ""} onClick={() => setView("workbench")}>
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>
          <span>Workbench</span>
        </button>
        <button className={view === "studio" ? "active" : ""} onClick={() => setView("studio")}>
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polygon points="16 3 21 8 8 21 3 21 3 16 16 3"/></svg>
          <span>Studio</span>
        </button>
        <button className={view === "store" ? "active" : ""} onClick={() => setView("store")}>
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="9" cy="21" r="1"/><circle cx="20" cy="21" r="1"/><path d="M1 1h4l2.68 13.39a2 2 0 0 0 2 1.61h9.72a2 2 0 0 0 2-1.61L23 6H6"/></svg>
          <span>Store</span>
        </button>
        <button className={view === "console" ? "active" : ""} onClick={() => setView("console")}>
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/></svg>
          <span>Console</span>
        </button>
      </nav>
      {view === "workbench" && renderWorkbench()}
      {view === "studio" && renderStudio()}
      {view === "store" && <div className="console-view"><div className="console-content"><BotStore /></div></div>}
      {view === "console" && renderConsole()}
      {showSettings && <Settings onClose={() => setShowSettings(false)} />}
      <StatusBar />
    </div>
  );
}

export default App;
