import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface TopologyNode {
  id: string;
  name: string;
  node_type: string;
  status: string;
}

const containerStyle: React.CSSProperties = {
  display: "flex",
  flexWrap: "wrap",
  gap: "12px",
  padding: "16px",
};

const nodeStyle: React.CSSProperties = {
  background: "#161b22",
  border: "1px solid #30363d",
  borderRadius: "8px",
  padding: "12px 16px",
  cursor: "pointer",
  minWidth: "160px",
};

export default function Topology() {
  const [nodes, setNodes] = useState<TopologyNode[]>([]);

  useEffect(() => {
    invoke<TopologyNode[]>("get_component_topology").then(setNodes).catch(() => {});
  }, []);

  const getNodeColor = (type: string) => {
    switch (type) {
      case "agent": return "#58a6ff";
      case "tool": return "#3fb950";
      case "model": return "#d29922";
      case "knowledge": return "#bc8cff";
      case "channel": return "#f0883e";
      default: return "#8b949e";
    }
  };

  return (
    <div>
      <h2 style={{ color: "#e6edf3", marginBottom: "16px" }}>Component Topology</h2>
      <div style={containerStyle}>
        {nodes.length === 0 && <p style={{ color: "#8b949e" }}>No components registered yet.</p>}
        {nodes.map((node) => (
          <div key={node.id} style={{ ...nodeStyle, borderLeft: `3px solid ${getNodeColor(node.node_type)}` }}>
            <div style={{ color: getNodeColor(node.node_type), fontSize: "11px", textTransform: "uppercase" }}>
              {node.node_type}
            </div>
            <div style={{ color: "#e6edf3", fontWeight: "bold", marginTop: "4px" }}>{node.name}</div>
            <div style={{ color: "#3fb950", fontSize: "12px", marginTop: "4px" }}>● {node.status}</div>
          </div>
        ))}
      </div>
      <div style={{ color: "#8b949e", fontSize: "12px", marginTop: "8px", padding: "0 16px" }}>
        Drag to reconnect | Click to inspect
      </div>
    </div>
  );
}