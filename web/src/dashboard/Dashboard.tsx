import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";

interface AgentEvent {
  id: string;
  session_id: string;
  event_type: string;
  source: string;
  data: any;
  timestamp: number;
}

interface TaskProgress {
  task_id: string;
  name: string;
  status: "running" | "completed" | "failed" | "pending";
  progress: number;
  started_at: number;
}

interface SystemResources {
  cpu_usage: number;
  memory_mb: number;
  memory_total_mb: number;
  uptime_seconds: number;
  active_agents: number;
  queue_depth: number;
}

const defaultResources: SystemResources = {
  cpu_usage: 0,
  memory_mb: 0,
  memory_total_mb: 16384,
  uptime_seconds: 0,
  active_agents: 0,
  queue_depth: 0,
};

const cardStyle: React.CSSProperties = {
  background: "#161b22",
  borderRadius: "8px",
  padding: "16px",
  border: "1px solid #30363d",
};

export function EventStreamView() {
  const [events, setEvents] = useState<AgentEvent[]>([]);
  const [filter, setFilter] = useState("");
  const endRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        const data = await invoke<AgentEvent[]>("get_recent_events", { limit: 50 });
        setEvents(data);
      } catch {
      }
    }, 2000);
    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    endRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [events]);

  const filtered = filter
    ? events.filter((e) => e.event_type.includes(filter) || e.source.includes(filter))
    : events;

  const formatTime = (ts: number) => {
    const d = new Date(ts);
    return `${d.getHours().toString().padStart(2, "0")}:${d.getMinutes().toString().padStart(2, "0")}:${d.getSeconds().toString().padStart(2, "0")}`;
  };

  const eventColor = (type: string) => {
    if (type.includes("error") || type.includes("failed")) return "#f85149";
    if (type.includes("completed")) return "#3fb950";
    if (type.includes("started") || type.includes("step")) return "#58a6ff";
    if (type.includes("approval")) return "#d29922";
    return "#8b949e";
  };

  return (
    <div style={cardStyle}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "12px" }}>
        <h3 style={{ color: "#e6edf3", margin: 0, fontSize: "14px", fontWeight: 600 }}>Event Stream</h3>
        <input
          type="text"
          placeholder="Filter events..."
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
          style={{
            background: "#0d1117",
            border: "1px solid #30363d",
            borderRadius: "4px",
            color: "#e6edf3",
            padding: "4px 8px",
            fontSize: "12px",
            width: "160px",
            outline: "none",
          }}
        />
      </div>
      <div style={{ maxHeight: "240px", overflowY: "auto", fontSize: "12px", fontFamily: "monospace" }}>
        {filtered.length === 0 && (
          <div style={{ color: "#8b949e", textAlign: "center", padding: "16px" }}>No events</div>
        )}
        {filtered.map((event) => (
          <div
            key={event.id}
            style={{
              display: "flex",
              gap: "8px",
              padding: "4px 0",
              borderBottom: "1px solid #21262d",
              alignItems: "center",
            }}
          >
            <span style={{ color: "#8b949e", minWidth: "64px" }}>{formatTime(event.timestamp)}</span>
            <span style={{ color: eventColor(event.event_type), minWidth: "120px" }}>{event.event_type}</span>
            <span style={{ color: "#6e7681", minWidth: "60px" }}>{event.source}</span>
            <span style={{ color: "#8b949e", flex: 1, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
              {JSON.stringify(event.data).slice(0, 80)}
            </span>
          </div>
        ))}
        <div ref={endRef} />
      </div>
    </div>
  );
}

export function TaskProgress() {
  const [tasks, setTasks] = useState<TaskProgress[]>([]);

  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        const data = await invoke<TaskProgress[]>("get_task_progress");
        setTasks(data);
      } catch {
      }
    }, 3000);
    return () => clearInterval(interval);
  }, []);

  const statusColor = (status: string) => {
    switch (status) {
      case "running": return "#58a6ff";
      case "completed": return "#3fb950";
      case "failed": return "#f85149";
      default: return "#8b949e";
    }
  };

  return (
    <div style={cardStyle}>
      <h3 style={{ color: "#e6edf3", margin: "0 0 12px 0", fontSize: "14px", fontWeight: 600 }}>Task Progress</h3>
      {tasks.length === 0 && (
        <div style={{ color: "#8b949e", textAlign: "center", padding: "16px", fontSize: "13px" }}>No active tasks</div>
      )}
      {tasks.map((task) => (
        <div key={task.task_id} style={{ marginBottom: "8px" }}>
          <div style={{ display: "flex", justifyContent: "space-between", fontSize: "12px", marginBottom: "4px" }}>
            <span style={{ color: "#e6edf3" }}>{task.name}</span>
            <span style={{ color: statusColor(task.status) }}>
              {task.status} {task.progress}%
            </span>
          </div>
          <div style={{ background: "#0d1117", borderRadius: "4px", height: "6px", overflow: "hidden" }}>
            <div
              style={{
                width: `${task.progress}%`,
                height: "100%",
                background: statusColor(task.status),
                borderRadius: "4px",
                transition: "width 0.5s ease",
              }}
            />
          </div>
        </div>
      ))}
    </div>
  );
}

export function SystemStatus() {
  const [resources, setResources] = useState<SystemResources>(defaultResources);

  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        const data = await invoke<SystemResources>("get_system_resources");
        setResources(data);
      } catch {
      }
    }, 5000);
    return () => clearInterval(interval);
  }, []);

  const memPercent = resources.memory_total_mb > 0
    ? Math.round((resources.memory_mb / resources.memory_total_mb) * 100)
    : 0;
  const uptimeHours = (resources.uptime_seconds / 3600).toFixed(1);

  const gaugeStyle = (_percent: number, _color: string): React.CSSProperties => ({
    background: "#0d1117",
    borderRadius: "4px",
    height: "8px",
    overflow: "hidden",
    marginTop: "4px",
  });

  const fillStyle = (percent: number, color: string): React.CSSProperties => ({
    width: `${Math.min(percent, 100)}%`,
    height: "100%",
    background: color,
    borderRadius: "4px",
    transition: "width 0.5s ease",
  });

  return (
    <div style={cardStyle}>
      <h3 style={{ color: "#e6edf3", margin: "0 0 12px 0", fontSize: "14px", fontWeight: 600 }}>System Resources</h3>
      <div style={{ fontSize: "12px", display: "flex", flexDirection: "column", gap: "10px" }}>
        <div>
          <div style={{ display: "flex", justifyContent: "space-between", color: "#8b949e" }}>
            <span>CPU</span>
            <span style={{ color: resources.cpu_usage > 80 ? "#f85149" : "#3fb950" }}>{resources.cpu_usage}%</span>
          </div>
          <div style={gaugeStyle(resources.cpu_usage, "#58a6ff")}>
            <div style={fillStyle(resources.cpu_usage, resources.cpu_usage > 80 ? "#f85149" : "#58a6ff")} />
          </div>
        </div>
        <div>
          <div style={{ display: "flex", justifyContent: "space-between", color: "#8b949e" }}>
            <span>Memory</span>
            <span>{resources.memory_mb} MB / {resources.memory_total_mb} MB ({memPercent}%)</span>
          </div>
          <div style={gaugeStyle(memPercent, "#3fb950")}>
            <div style={fillStyle(memPercent, memPercent > 85 ? "#f85149" : "#3fb950")} />
          </div>
        </div>
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "8px", marginTop: "4px" }}>
          <div style={{ color: "#8b949e" }}>
            <div>Uptime</div>
            <div style={{ color: "#e6edf3", fontWeight: 600 }}>{uptimeHours}h</div>
          </div>
          <div style={{ color: "#8b949e" }}>
            <div>Active Agents</div>
            <div style={{ color: "#58a6ff", fontWeight: 600 }}>{resources.active_agents}</div>
          </div>
          <div style={{ color: "#8b949e" }}>
            <div>Queue Depth</div>
            <div style={{ color: "#d29922", fontWeight: 600 }}>{resources.queue_depth}</div>
          </div>
        </div>
      </div>
    </div>
  );
}

export default function Dashboard() {
  return (
    <div>
      <h2 style={{ color: "#e6edf3", marginBottom: "16px", fontSize: "18px" }}>Agent Runtime Dashboard</h2>
      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "12px" }}>
        <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
          <EventStreamView />
          <TaskProgress />
        </div>
        <div>
          <SystemStatus />
        </div>
      </div>
    </div>
  );
}