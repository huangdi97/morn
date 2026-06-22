import React, { useState, useRef, useEffect } from "react";
import { api } from "./api";
import { ErrorBoundary } from "./components/ErrorBoundary";
import { ComponentEditor } from "./studio/ComponentEditor";
import { AgentBuilder } from "./studio/AgentBuilder";
import { TeamBuilder } from "./studio/TeamBuilder";
import { TestPanel } from "./studio/TestPanel";
import WorkflowBuilder from "./studio/WorkflowBuilder";
import { TeamTemplateSelector } from "./studio/TeamTemplateSelector";
import { DevZone } from "./studio/DevZone";
import { ComponentTypeManager } from "./studio/ComponentTypeManager";
import { McpManager } from "./studio/McpManager";
import { QuickActions } from "./QuickActions";
import Topology from "./console/Topology";
import SystemInfo from "./console/SystemInfo";
import AdminDashboard from "./console/AdminDashboard";
import CostCenter from "./console/CostCenter";
import ReliabilityPanel from "./console/ReliabilityPanel";
import RoiCalculator from "./console/RoiCalculator";
import SystemCheck from "./console/SystemCheck";
import Governance from "./console/Governance";
import Security from "./console/Security";
import Hub from "./console/Hub";
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
import CreatorEarnings from "./console/CreatorEarnings";
import GitPanel from "./console/GitPanel";
import OAuthPanel from "./console/OAuthPanel";
import PluginManagerPanel from "./console/PluginManagerPanel";
import CreatePluginWizard from "./plugins/CreatePluginWizard";
import BotStore from "./store/BotStore";
import { Settings } from "./Settings";
import { OnboardingWizard } from './OnboardingWizard';
import { FeatureGuide } from './FeatureGuide';
import StatusBar from "./StatusBar";
import PipelineFlow from "./components/PipelineFlow";
import { ToastItem } from "./components/Toast";
import { LocaleProvider, useTranslation } from "./i18n";
import VoiceInput from "./components/VoiceInput";
import { CommandPalette } from "./components/CommandPalette";
import { DesignSystem } from "./design/DesignSystem";
import { RenderSlot } from "./slots/RenderSlot";
import "./styles/base.css";
import "./styles/skeleton.css";
import "./styles/dashboard.css";
import "./styles/chat.css";
import "./styles/studio.css";
import "./styles/console.css";

type View = "workbench" | "studio" | "hub" | "console" | "design";

interface Message {
  role: "user" | "assistant";
  content: string;
  timestamp: number;
}

const CHAT_KEY = "morn_chat_history";
const THEME_KEY = "morn-theme";

function App() {
  return (
    <LocaleProvider>
      <AppInner />
    </LocaleProvider>
  );
}

function AppInner() {
  const { t } = useTranslation();
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
  const [showOnboarding, setShowOnboarding] = useState(
    () => !localStorage.getItem('morn_onboarding_done')
  );
  const [showGuide, setShowGuide] = useState(false);
  const [sendingIndex, setSendingIndex] = useState<number | null>(null);
  const [loading, setLoading] = useState<Record<string, boolean>>({ workbench: true, studio: true, console: true });
  const [workStep, setWorkStep] = useState(0);
  const [workVisible, setWorkVisible] = useState(false);
  const [workLogs, setWorkLogs] = useState<any[]>([]);
  const [hubAvailable, setHubAvailable] = useState(true);
  const workLogsEndRef = useRef<HTMLDivElement>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const abortControllerRef = useRef<AbortController | null>(null);
  const chatAreaRef = useRef<HTMLDivElement>(null);
  const [showScrollBtn, setShowScrollBtn] = useState(false);
  const [feedback, setFeedback] = useState<Record<string, "like" | "dislike">>(() => {
    try {
      const saved = localStorage.getItem("morn_feedback");
      return saved ? JSON.parse(saved) : {};
    } catch {
      return {};
    }
  });
  const mainTabsRef = useRef<HTMLDivElement>(null);
  const [indicatorStyle, setIndicatorStyle] = useState({ left: 0, width: 0 });
  const [confirmClear, setConfirmClear] = useState(false);
  const confirmTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [showCommandPalette, setShowCommandPalette] = useState(false);
  const [toasts, setToasts] = useState<Array<{ id: number; type: "success" | "error" | "info" | "warning"; message: string }>>([]);
  const toastIdRef = useRef(0);

  const showToast = (type: "success" | "error" | "info" | "warning", message: string) => {
    const id = ++toastIdRef.current;
    setToasts(prev => [...prev, { id, type, message }]);
  };

  const removeToast = (id: number) => {
    setToasts(prev => prev.filter(t => t.id !== id));
  };

  useEffect(() => {
    const nav = mainTabsRef.current;
    if (!nav) return;
    const activeBtn = nav.querySelector('button.active') as HTMLElement;
    if (activeBtn) {
      setIndicatorStyle({
        left: activeBtn.offsetLeft,
        width: activeBtn.offsetWidth,
      });
    }
  }, [view]);

  useEffect(() => {
    localStorage.setItem(CHAT_KEY, JSON.stringify(messages));
  }, [messages]);

  useEffect(() => {
    localStorage.setItem(THEME_KEY, theme);
  }, [theme]);

  useEffect(() => {
    localStorage.setItem("morn_feedback", JSON.stringify(feedback));
  }, [feedback]);

  useEffect(() => {
    const el = chatAreaRef.current;
    if (!el) return;
    const handleScroll = () => {
      const dist = el.scrollHeight - el.scrollTop - el.clientHeight;
      setShowScrollBtn(dist > 200);
    };
    el.addEventListener("scroll", handleScroll);
    return () => el.removeEventListener("scroll", handleScroll);
  }, []);

  useEffect(() => {
    const handler = (e: StorageEvent) => {
      if (e.key === 'morn-theme') setTheme(e.newValue || 'cyber');
    };
    window.addEventListener('storage', handler);
    return () => window.removeEventListener('storage', handler);
  }, []);

  useEffect(() => {
    api.getStatus().then((s: any) => {
      setStatus(t('status.version', { version: s.version, turns: s.turn_count }));
    });
  }, []);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, isTyping]);

  useEffect(() => {
    const poll = async () => {
      try {
        const logs = await api.getRecentLogs();
        setWorkLogs(logs);
      } catch { /* ignore */ }
    };
    poll();
    const interval = setInterval(poll, 5000);
    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    workLogsEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [workLogs]);

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
    if (workLogs.length > 0) {
      setWorkVisible(true);
    }
  }, [workLogs]);

  useEffect(() => {
    (async () => {
      try {
        const { invoke } = await import("@tauri-apps/api/core");
        const plugins = await invoke<{id: string; enabled: boolean}[]>("list_morn_plugins");
        const hub = plugins.find(p => p.id === "morn:hub");
        setHubAvailable(hub?.enabled ?? true);
      } catch { /* 非 Tauri 环境默认显示 */ }
    })();
  }, []);

  useEffect(() => {
    if (view !== "hub") {
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
      setStatus(t('status.version', { version: s.version, turns: s.turn_count }));
      showToast("info", t('chat.clear_toast'));
      return;
    }

    const userMsg: Message = { role: "user", content: text, timestamp: Date.now() };
    setMessages((prev) => {
      const next = [...prev, userMsg];
      setSendingIndex(next.length - 1);
      return next;
    });
    setIsTyping(true);
    setShowScrollBtn(false);

    const controller = new AbortController();
    abortControllerRef.current = controller;

    try {
      const response = await api.sendMessage(text, controller.signal);
      const assistantMsg: Message = { role: "assistant", content: response.text ?? response, timestamp: Date.now() };
      setMessages((prev) => [...prev, assistantMsg]);

      if (response.execution_events) {
        setWorkLogs(response.execution_events);
      } else {
        // fallback: poll for logs
        const logs = await api.getRecentLogs();
        setWorkLogs(logs);
      }

      const s: any = await api.getStatus();
      setStatus(t('status.version', { version: s.version, turns: s.turn_count }));

      // keep workLogs visible for 30s after response
      setWorkVisible(true);
      setTimeout(() => setWorkVisible(false), 30000);
    } catch (e: any) {
      if (e instanceof DOMException && e.name === "AbortError") return;
      const errorMsg: Message = {
        role: "assistant",
        content: `Error: ${e}`,
        timestamp: Date.now(),
      };
      setMessages((prev) => [...prev, errorMsg]);
    } finally {
      setIsTyping(false);
      abortControllerRef.current = null;
      setTimeout(() => setSendingIndex(null), 500);
    }
  };

  const clearHistory = async () => {
    await api.clearHistory();
    setMessages([]);
    const s: any = await api.getStatus();
    setStatus(t('status.version', { version: s.version, turns: s.turn_count }));
    showToast("info", t('chat.clear_toast'));
  };

  const handleClearClick = () => {
    if (confirmClear) {
      clearHistory();
      setConfirmClear(false);
      if (confirmTimerRef.current) clearTimeout(confirmTimerRef.current);
    } else {
      setConfirmClear(true);
      confirmTimerRef.current = setTimeout(() => setConfirmClear(false), 3000);
    }
  };

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      const target = e.target as HTMLElement;
      if (confirmClear && !target.closest('.clear-btn')) {
        setConfirmClear(false);
        if (confirmTimerRef.current) clearTimeout(confirmTimerRef.current);
      }
    };
    document.addEventListener('click', handler);
    return () => document.removeEventListener('click', handler);
  }, [confirmClear]);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        setShowCommandPalette(prev => !prev);
      }
    };
    document.addEventListener('keydown', handler);
    return () => document.removeEventListener('keydown', handler);
  }, []);

  const commandPaletteViews = [
    { key: 'workbench', label: '打开 Workbench', icon: '💬' },
    { key: 'studio', label: '打开 Studio', icon: '🎨' },
    { key: 'hub', label: '打开 Store', icon: '🏪' },
    { key: 'console', label: '打开 Console', icon: '📋' },
    ...(import.meta.env.DEV ? [{ key: 'design', label: '打开 Design System', icon: '🎨' }] : []),
  ];

  const sendQuickAction = async (text: string) => {
    setInput(text);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  };

  const handleStop = () => {
    abortControllerRef.current?.abort();
  };

  const retryMessage = (msgIndex: number) => {
    for (let i = msgIndex - 1; i >= 0; i--) {
      if (messages[i].role === "user") {
        const text = messages[i].content;
        setInput(text);
        setTimeout(() => sendMessage(), 0);
        return;
      }
    }
  };

  const scrollToBottom = () => {
    chatAreaRef.current?.scrollTo({ top: chatAreaRef.current.scrollHeight, behavior: "smooth" });
  };

  const toggleFeedback = (msgIndex: number, type: "like" | "dislike") => {
    const key = `morn_feedback_${msgIndex}`;
    setFeedback((prev) => {
      if (prev[key] === type) {
        const next = { ...prev };
        delete next[key];
        return next;
      }
      return { ...prev, [key]: type };
    });
  };

  const isError = (content: string) => content.startsWith("Error: ");

  const formatTime = (ts: number) => {
    const d = new Date(ts);
    const hh = String(d.getHours()).padStart(2, "0");
    const mm = String(d.getMinutes()).padStart(2, "0");
    return `${hh}:${mm}`;
  };

  const [studioTab, setStudioTab] = useState<"editor" | "builder" | "test" | "teams" | "team" | "dev" | "types" | "mcp" | "create_plugin">("builder");
  const [consoleTab, setConsoleTab] = useState<"dashboard" | "journey" | "topology" | "system" | "cost" | "roi" | "reliability" | "governance" | "security" | "hub" | "system_check" | "notifications" | "memory" | "connections" | "audio" | "cost_tracking" | "local_models" | "analytics" | "sandbox" | "proactive" | "business" | "earnings" | "git" | "plugins" | "oauth">("dashboard");
  const [workbenchTab, setWorkbenchTab] = useState<"chat" | "workflow">("chat");

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
        {[1, 2, 3, 4, 5, 6, 7, 8].map(i => <div key={i} className="skeleton skeleton-studio-nav-item" />)}
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
    { name: "Planner", active: true, key: "planner" },
    { name: "Coder", active: true, key: "coder" },
    { name: "Reviewer", active: true, key: "reviewer" },
    { name: "Tester", active: true, key: "tester" },
    { name: "Monitor", active: true, key: "monitor" },
    { name: "Deployer", active: false, key: "deployer" },
    { name: "Optimizer", active: false, key: "optimizer" },
    { name: "Analyst", active: false, key: "analyst" },
  ];

  function AgentBar({ isTyping }: { isTyping: boolean }) {
    const maxVisible = 6;
    const visible = AGENTS.slice(0, maxVisible);
    const extra = AGENTS.length - maxVisible;

    return (
      <div className="agent-bar">
        {visible.map((agent, i) => (
          <span key={agent.name} className="agent-item">
            <span
              className={`agent-dot ${agent.active ? "active" : "inactive"}${agent.active && isTyping ? " typing" : ""}`}
              style={agent.active && isTyping ? { animationDelay: `${i * 0.2}s` } : {}}
            />
            {t('agent.'+agent.key)}
          </span>
        ))}
        {extra > 0 && <span className="agent-extra">{t('agent.more', { count: extra })}</span>}
      </div>
    );
  }

  const renderStudio = () => (
    <div className="studio-view">
      <nav className="studio-tabs">
        <button className={studioTab === "editor" ? "active" : ""} onClick={() => setStudioTab("editor")}>{t('studio_tab.component_editor')}</button>
        <button className={studioTab === "builder" ? "active" : ""} onClick={() => setStudioTab("builder")}>{t('studio_tab.agent_builder')}</button>
        <button className={studioTab === "teams" ? "active" : ""} onClick={() => setStudioTab("teams")}>{t('studio_tab.teams')}</button>
        <button className={studioTab === "team" ? "active" : ""} onClick={() => setStudioTab("team")}>{t('studio_tab.team_builder')}</button>
        <button className={studioTab === "dev" ? "active" : ""} onClick={() => setStudioTab("dev")}>{t('studio_tab.dev')}</button>
        <button className={studioTab === "types" ? "active" : ""} onClick={() => setStudioTab("types")}>{t('studio_tab.types')}</button>
        <button className={studioTab === "mcp" ? "active" : ""} onClick={() => setStudioTab("mcp")}>{t('studio_tab.mcp')}</button>
        <button className={studioTab === "create_plugin" ? "active" : ""} onClick={() => setStudioTab("create_plugin")}>{t('studio_tab.create_plugin') || 'Create Plugin'}</button>
        <button className={studioTab === "test" ? "active" : ""} onClick={() => setStudioTab("test")}>{t('studio_tab.test_runner')}</button>
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
                      showToast("success", `团队 "${template.name}" 创建成功`);
                    } catch (e: any) {
                      showToast("error", `创建失败: ${e}`);
                    }
                  }}
              />
            )}
            {studioTab === "team" && <TeamBuilder />}
            {studioTab === "test" && <TestPanel />}
            {studioTab === "dev" && <DevZone />}
            {studioTab === "types" && <ComponentTypeManager />}
            {studioTab === "mcp" && <McpManager />}
            {studioTab === "create_plugin" && <CreatePluginWizard />}
          </>
        )}
      </div>
      <RenderSlot name="studio-tools" />
    </div>
  );

  const renderConsole = () => (
    <div className="console-view">
      <nav className="console-tabs">
        <button className={consoleTab === "dashboard" ? "active" : ""} onClick={() => setConsoleTab("dashboard")}>{t('console_tab.dashboard')}</button>
        <button className={consoleTab === "journey" ? "active" : ""} onClick={() => setConsoleTab("journey")}>{t('console_tab.journey')}</button>
        <button className={consoleTab === "topology" ? "active" : ""} onClick={() => setConsoleTab("topology")}>{t('console_tab.topology')}</button>
        <button className={consoleTab === "system" ? "active" : ""} onClick={() => setConsoleTab("system")}>{t('console_tab.system')}</button>
        <button className={consoleTab === "cost" ? "active" : ""} onClick={() => setConsoleTab("cost")}>{t('console_tab.cost')}</button>
        <button className={consoleTab === "roi" ? "active" : ""} onClick={() => setConsoleTab("roi")}>{t('console_tab.roi')}</button>
        <button className={consoleTab === "reliability" ? "active" : ""} onClick={() => setConsoleTab("reliability")}>{t('console_tab.reliability')}</button>
        <button className={consoleTab === "governance" ? "active" : ""} onClick={() => setConsoleTab("governance")}>{t('console_tab.governance')}</button>
        <button className={consoleTab === "security" ? "active" : ""} onClick={() => setConsoleTab("security")}>{t('console_tab.security')}</button>
        <button className={consoleTab === "hub" ? "active" : ""} onClick={() => setConsoleTab("hub")}>{t('console_tab.hub')}</button>
        <button className={consoleTab === "system_check" ? "active" : ""} onClick={() => setConsoleTab("system_check")}>{t('console_tab.self_check')}</button>
        <button className={consoleTab === "notifications" ? "active" : ""} onClick={() => setConsoleTab("notifications")}>{t('console_tab.notifications')}</button>
        <button className={consoleTab === "memory" ? "active" : ""} onClick={() => setConsoleTab("memory")}>{t('console_tab.memory')}</button>
        <button className={consoleTab === "connections" ? "active" : ""} onClick={() => setConsoleTab("connections")}>{t('console_tab.connections')}</button>
        <button className={consoleTab === "audio" ? "active" : ""} onClick={() => setConsoleTab("audio")}>{t('console_tab.audio')}</button>
        <button className={consoleTab === "cost_tracking" ? "active" : ""} onClick={() => setConsoleTab("cost_tracking")}>{t('console_tab.cost_tracking')}</button>
        <button className={consoleTab === "local_models" ? "active" : ""} onClick={() => setConsoleTab("local_models")}>{t('console_tab.local_models')}</button>
        <button className={consoleTab === "analytics" ? "active" : ""} onClick={() => setConsoleTab("analytics")}>{t('console_tab.analytics')}</button>
        <button className={consoleTab === "sandbox" ? "active" : ""} onClick={() => setConsoleTab("sandbox")}>{t('console_tab.sandbox')}</button>
        <button className={consoleTab === "proactive" ? "active" : ""} onClick={() => setConsoleTab("proactive")}>{t('console_tab.proactive')}</button>
        <button className={consoleTab === "business" ? "active" : ""} onClick={() => setConsoleTab("business")}>{t('console_tab.business')}</button>
        <button className={consoleTab === "earnings" ? "active" : ""} onClick={() => setConsoleTab("earnings")}>{t('console_tab.earnings')}</button>
        <button className={consoleTab === "git" ? "active" : ""} onClick={() => setConsoleTab("git")}>{t('console_tab.git')}</button>
        <button className={consoleTab === "plugins" ? "active" : ""} onClick={() => setConsoleTab("plugins")}>{t('console_tab.plugins')}</button>
        <button className={consoleTab === "oauth" ? "active" : ""} onClick={() => setConsoleTab("oauth")}>{t('console_tab.oauth')}</button>
      </nav>
      <div className="console-content">
        {loading.console ? <SkeletonConsole /> : (
          <>
            {consoleTab === "dashboard" && <AdminDashboard onNavigate={(tab) => setConsoleTab(tab as any)} />}
            {consoleTab === "journey" && <UserJourney />}
            {consoleTab === "topology" && <Topology />}
            {consoleTab === "system" && <SystemInfo />}
            {consoleTab === "cost" && <CostCenter />}
            {consoleTab === "roi" && <RoiCalculator />}
            {consoleTab === "reliability" && <ReliabilityPanel />}
            {consoleTab === "governance" && <Governance />}
            {consoleTab === "security" && <Security />}
            {consoleTab === "hub" && <Hub />}
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
            {consoleTab === "earnings" && <CreatorEarnings />}
            {consoleTab === "git" && <GitPanel />}
            {consoleTab === "plugins" && <PluginManagerPanel />}
            {consoleTab === "oauth" && <OAuthPanel />}
          </>
        )}
      </div>
      <RenderSlot name="console-panels" />
    </div>
  );

  const renderWorkbench = () => (
    <>
      <header className="header">
        <h1>{t('app.title')}</h1>
        <span className="status">{status}</span>
        <button className={`clear-btn${confirmClear ? ' confirming' : ''}`} onClick={handleClearClick}>
          {confirmClear ? t('chat.clear_confirm') : t('chat.clear')}
        </button>
        <button className="settings-btn" onClick={() => setShowSettings(true)}>
          ⚙
        </button>
      </header>

      <AgentBar isTyping={isTyping} />
      <PipelineFlow logs={workLogs} visible={workVisible} />

      <nav className="workbench-tabs">
        <button className={workbenchTab === "chat" ? "active" : ""} onClick={() => setWorkbenchTab("chat")}>{t('workbench_tab.chat')}</button>
        <button className={workbenchTab === "workflow" ? "active" : ""} onClick={() => setWorkbenchTab("workflow")}>{t('workbench_tab.workflow')}</button>
      </nav>

      {workbenchTab === "chat" ? (
        <>
      <main className="chat-area" ref={chatAreaRef}>
        {loading.workbench && messages.length === 0 ? <SkeletonChat /> : (
          <>
        {messages.length === 0 && (
          <div style={{ textAlign: "center", padding: "40px 20px", color: "var(--text-secondary)" }}>
            <div style={{ fontSize: "48px", marginBottom: "12px" }}>🤖</div>
            <h2 style={{ color: "var(--text-primary)", margin: "0 0 8px 0" }}>{t('welcome.title')}</h2>
            <p style={{ fontSize: "14px", margin: "0 0 24px 0" }}>{t('welcome.description')}</p>
            <QuickActions onSend={sendQuickAction} />
          </div>
        )}
        {messages.map((msg, i) => {
          const showSeparator = i > 0 && (msg.timestamp - messages[i - 1].timestamp) > 5 * 60 * 1000;
          const isErr = isError(msg.content);
          const feedbackKey = `morn_feedback_${i}`;
          const fb = feedback[feedbackKey];
          return (
            <React.Fragment key={i}>
              {showSeparator && <div className="time-separator">{formatTime(msg.timestamp)}</div>}
              <div
                className={`message ${msg.role}${i === sendingIndex ? ' sending' : ''}${isErr ? ' error' : ''}`}
                style={{"--msg-index": i} as React.CSSProperties}
              >
                <div className="avatar">{isErr ? "⚠️" : msg.role === "user" ? "👤" : "🤖"}</div>
                <div>
                  <div className="bubble">
                    <div className="bubble-text">{msg.content}</div>
                    <span className="timestamp">{formatTime(msg.timestamp)}</span>
                  </div>
                  {isErr && (
                    <button className="retry-btn" onClick={() => retryMessage(i)} title={t('chat.retry')}>↻</button>
                  )}
                  {msg.role === "assistant" && !isErr && (
                    <div className="feedback-btns">
                      <button
                        className={`feedback-btn${fb === "like" ? " liked" : ""}`}
                        onClick={() => toggleFeedback(i, "like")}
                      >
                        👍
                      </button>
                      <button
                        className={`feedback-btn${fb === "dislike" ? " disliked" : ""}`}
                        onClick={() => toggleFeedback(i, "dislike")}
                      >
                        👎
                      </button>
                    </div>
                  )}
                </div>
              </div>
            </React.Fragment>
          );
        })}
        {isTyping && (
          <div className="message assistant">
            <div className="avatar">🤖</div>
            <div className="bubble typing-indicator">
              <span></span><span></span><span></span>
            </div>
          </div>
        )}
        <div ref={messagesEndRef} />
        {showScrollBtn && (
          <button className="scroll-bottom-btn" onClick={scrollToBottom}>↓</button>
        )}
          </>
        )}
      </main>

      <footer className="input-bar">
        <VoiceInput onTranscribed={(text) => setInput(text)} />
        <div className="input-wrap">
        <textarea
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={t('chat.placeholder')}
          rows={1}
        />
        </div>
        {isTyping ? (
          <button className="stop-btn" onClick={handleStop}>■ {t('chat.stop')}</button>
        ) : (
          <button onClick={(e) => { (e.currentTarget as HTMLElement).classList.add('pressing'); setTimeout(() => (e.currentTarget as HTMLElement).classList.remove('pressing'), 250); sendMessage(); }} disabled={!input.trim()}>
            {t('chat.send')}
          </button>
        )}
      </footer>
        </>
      ) : (
        <div className="workbench-workflow">
          <WorkflowBuilder />
        </div>
      )}
    </>
  );

  return (
    <div className="app" data-theme={theme}>
      <nav className="main-tabs" ref={mainTabsRef}>
        <button className={view === "workbench" ? "active" : ""} onClick={() => setView("workbench")} data-tooltip={t('nav.workbench_tooltip')} data-guide-target="workbench">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>
          <span>{t('nav.workbench')}</span>
        </button>
        <button className={view === "studio" ? "active" : ""} onClick={() => setView("studio")} data-tooltip={t('nav.studio_tooltip')} data-guide-target="studio">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polygon points="16 3 21 8 8 21 3 21 3 16 16 3"/></svg>
          <span>{t('nav.studio')}</span>
        </button>
        {hubAvailable && (
        <button className={view === "hub" ? "active" : ""} onClick={() => setView("hub")} data-tooltip={t('nav.hub_tooltip')} data-guide-target="hub">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="9" cy="21" r="1"/><circle cx="20" cy="21" r="1"/><path d="M1 1h4l2.68 13.39a2 2 0 0 0 2 1.61h9.72a2 2 0 0 0 2-1.61L23 6H6"/></svg>
          <span>{t('nav.hub')}</span>
        </button>
        )}
        <button className={view === "console" ? "active" : ""} onClick={() => setView("console")} data-tooltip={t('nav.console_tooltip')} data-guide-target="console">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/></svg>
          <span>{t('nav.console')}</span>
        </button>
        {import.meta.env.DEV && (
        <button className={view === "design" ? "active" : ""} onClick={() => setView("design")} data-tooltip="Design System">
          <span style={{ fontSize: 18 }}>🎨</span>
          <span>Design</span>
        </button>
        )}
      <div className="main-tab-indicator" style={{ left: indicatorStyle.left, width: indicatorStyle.width }} />
        </nav>
        <ErrorBoundary onRetry={() => api.retryLastOperation()}>{view === "workbench" && renderWorkbench()}</ErrorBoundary>
        <ErrorBoundary onRetry={() => api.retryLastOperation()}>{view === "studio" && renderStudio()}</ErrorBoundary>
        <ErrorBoundary onRetry={() => api.retryLastOperation()}>{hubAvailable && view === "hub" && <div className="console-view"><div className="console-content"><BotStore /></div><RenderSlot name="store-tabs" /></div>}</ErrorBoundary>
        <ErrorBoundary onRetry={() => api.retryLastOperation()}>{view === "console" && renderConsole()}</ErrorBoundary>
        {import.meta.env.DEV && <ErrorBoundary onRetry={() => api.retryLastOperation()}>{view === "design" && <div className="console-view"><div className="console-content"><DesignSystem /></div></div>}</ErrorBoundary>}
      {showSettings && <Settings onClose={() => setShowSettings(false)} showToast={showToast} />}
      <StatusBar />
      {showOnboarding && (
        <OnboardingWizard
          onComplete={() => {
            localStorage.setItem('morn_onboarding_done', 'true');
            setShowOnboarding(false);
            setShowGuide(true);
          }}
        />
      )}
      {showGuide && (
        <FeatureGuide
          onComplete={() => setShowGuide(false)}
          onSkip={() => setShowGuide(false)}
        />
      )}
      {showCommandPalette && (
        <CommandPalette
          views={commandPaletteViews}
          onNavigate={(v) => setView(v as View)}
          onClose={() => setShowCommandPalette(false)}
        />
      )}
      <div className="toast-container">
        {toasts.map(t => (
          <ToastItem key={t.id} toast={t} onRemove={removeToast} />
        ))}
      </div>
    </div>
  );
}

export default App;
