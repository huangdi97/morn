import { useState, useEffect } from "react";
import { api } from "../api";

interface SystemInfoData {
  version: string;
  cpu_usage: number;
  memory_used_mb: number;
  memory_total_mb: number;
  disk_free_mb: number;
  os: string;
  uptime_secs: number;
}

const cardStyle: React.CSSProperties = {
  background: "#161b22",
  borderRadius: "8px",
  padding: "16px",
  border: "1px solid #30363d",
};

export default function SystemInfo() {
  const [info, setInfo] = useState<SystemInfoData>({
    version: "0.1.0",
    cpu_usage: 0,
    memory_used_mb: 0,
    memory_total_mb: 8192,
    disk_free_mb: 50000,
    os: "linux",
    uptime_secs: 0,
  });

  useEffect(() => {
    api.getSystemStatus().then((res: { dashboard: any; system_info: SystemInfoData }) => {
      setInfo(res.system_info);
    }).catch(() => {});
  }, []);

  const memoryPercent = info.memory_total_mb > 0 ? (info.memory_used_mb / info.memory_total_mb) * 100 : 0;
  const diskPercent = (info.disk_free_mb / 100000) * 100;
  const uptimeHours = (info.uptime_secs / 3600).toFixed(1);

  const Bar = ({ percent, color }: { percent: number; color: string }) => (
    <div style={{ background: "#21262d", borderRadius: "4px", height: "8px", overflow: "hidden", marginTop: "4px" }}>
      <div style={{ width: `${Math.min(percent, 100)}%`, height: "100%", background: color, borderRadius: "4px" }} />
    </div>
  );

  return (
    <div>
      <h2 style={{ color: "#e6edf3", marginBottom: "16px" }}>System</h2>

      <div style={{ display: "grid", gridTemplateColumns: "repeat(2, 1fr)", gap: "12px" }}>
        <div style={cardStyle}>
          <div style={{ color: "#8b949e", fontSize: "13px" }}>Version</div>
          <div style={{ color: "#e6edf3", fontSize: "18px", fontWeight: "bold", marginTop: "8px" }}>Morn v{info.version}</div>
        </div>
        <div style={cardStyle}>
          <div style={{ color: "#8b949e", fontSize: "13px" }}>Operating System</div>
          <div style={{ color: "#e6edf3", fontSize: "18px", fontWeight: "bold", marginTop: "8px", textTransform: "capitalize" }}>{info.os}</div>
        </div>
        <div style={cardStyle}>
          <div style={{ color: "#8b949e", fontSize: "13px" }}>CPU Usage</div>
          <div style={{ color: "#58a6ff", fontSize: "18px", fontWeight: "bold", marginTop: "8px" }}>{info.cpu_usage}%</div>
          <Bar percent={info.cpu_usage} color="#58a6ff" />
        </div>
        <div style={cardStyle}>
          <div style={{ color: "#8b949e", fontSize: "13px" }}>Memory</div>
          <div style={{ color: "#d29922", fontSize: "18px", fontWeight: "bold", marginTop: "8px" }}>
            {info.memory_used_mb} MB / {info.memory_total_mb} MB
          </div>
          <Bar percent={memoryPercent} color="#d29922" />
        </div>
        <div style={cardStyle}>
          <div style={{ color: "#8b949e", fontSize: "13px" }}>Free Disk</div>
          <div style={{ color: "#3fb950", fontSize: "18px", fontWeight: "bold", marginTop: "8px" }}>
            {info.disk_free_mb} MB
          </div>
          <Bar percent={diskPercent} color="#3fb950" />
        </div>
        <div style={cardStyle}>
          <div style={{ color: "#8b949e", fontSize: "13px" }}>Uptime</div>
          <div style={{ color: "#bc8cff", fontSize: "18px", fontWeight: "bold", marginTop: "8px" }}>{uptimeHours} hours</div>
        </div>
      </div>
    </div>
  );
}
