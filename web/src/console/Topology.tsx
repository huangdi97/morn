import { useEffect, useMemo, useRef, useState, type CSSProperties, type PointerEvent } from "react";
import { useTranslation } from '../i18n';
import { api } from "../api";

interface TopologyNode { id: string; name: string; node_type: string; status: string; execution_id?: string; }
interface TopologyEdge { id: string; from: string; to: string; label?: string; }
interface NodePosition { x: number; y: number; }
interface ExecutionStatus { node_id: string; status: string; execution_id: string; }

const nodeWidth = 180; const nodeHeight = 82;

const stageStyle: CSSProperties = { position: "relative", minHeight: "440px", border: "1px solid #30363d", borderRadius: "8px", background: "#0d1117", overflow: "hidden" };
const nodeStyle: CSSProperties = { position: "absolute", width: `${nodeWidth}px`, height: `${nodeHeight}px`, background: "#161b22", border: "1px solid #30363d", borderRadius: "8px", padding: "12px 14px", cursor: "grab", userSelect: "none", boxSizing: "border-box" };

function defaultPosition(index: number): NodePosition {
  const columns = 3;
  return { x: 24 + (index % columns) * 230, y: 24 + Math.floor(index / columns) * 132 };
}

function getNodeColor(type: string) {
  switch (type) { case "agent": return "#58a6ff"; case "tool": return "#3fb950"; case "model": return "#d29922"; case "knowledge": return "#bc8cff"; case "channel": return "#f0883e"; case "capability": return "#79c0ff"; default: return "#8b949e"; }
}

function getStatusColor(status: string) {
  switch (status) {
    case "running": return "#3b82f6";
    case "completed": case "success": return "#22c55e";
    case "failed": case "error": return "#ef4444";
    case "queued": return "#8b949e";
    default: return "#6b7280";
  }
}

function getStatusGlow(status: string): string {
  switch (status) {
    case "running": return "0 0 12px rgba(59,130,246,0.5)";
    case "failed": case "error": return "0 0 12px rgba(239,68,68,0.5)";
    default: return "none";
  }
}

export default function Topology() {
  const { t } = useTranslation();
  const [nodes, setNodes] = useState<TopologyNode[]>([]);
  const [positions, setPositions] = useState<Record<string, NodePosition>>({});
  const [edges, setEdges] = useState<TopologyEdge[]>([]);
  const [dragging, setDragging] = useState<{ id: string; offsetX: number; offsetY: number } | null>(null);
  const [selectedEdgeId, setSelectedEdgeId] = useState<string | null>(null);
  const [executionStatus, setExecutionStatus] = useState<Record<string, ExecutionStatus>>({});
  const stageRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const fetchTopology = () => {
      api.getComponentTopology().then((result: TopologyNode[]) => {
        setNodes(result);
        setPositions((current) => {
          const next = { ...current };
          result.forEach((node, index) => { if (!next[node.id]) next[node.id] = defaultPosition(index); });
          return next;
        });
        setEdges((current) => {
          if (current.length > 0 || result.length < 2) return current;
          return result.slice(1).map((node, index) => ({ id: `${result[index].id}->${node.id}`, from: result[index].id, to: node.id, label: "data" }));
        });
      }).catch(() => {});
    };
    fetchTopology();
    const topoInterval = setInterval(fetchTopology, 15000);
    return () => clearInterval(topoInterval);
  }, []);

  useEffect(() => {
    const fetchExecutionStatus = () => {
      api.getAgentPoolStatus().then((data: any) => {
        if (!data?.tasks) return;
        const statusMap: Record<string, ExecutionStatus> = {};
        for (const task of data.tasks) {
          const matchedNode = nodes.find((n) => n.name === task.agent_id || n.id === task.agent_id);
          if (matchedNode) {
            statusMap[matchedNode.id] = { node_id: matchedNode.id, status: task.status, execution_id: task.id };
            if (task.status === "running") {
              // animation state tracked by executionStatus
            }
          }
        }
        setExecutionStatus(statusMap);
        setNodes((current) =>
          current.map((node) => {
            const exec = statusMap[node.id];
            if (exec) {
              return { ...node, status: exec.status, execution_id: exec.execution_id };
            }
            return node;
          }),
        );
      }).catch(() => {});
    };
    if (nodes.length > 0) {
      fetchExecutionStatus();
      const execInterval = setInterval(fetchExecutionStatus, 3000);
      return () => clearInterval(execInterval);
    }
  }, [nodes]);

  const canvasSize = useMemo(() => {
    const values = Object.values(positions);
    const maxX = Math.max(760, ...values.map((pos) => pos.x + nodeWidth + 24));
    const maxY = Math.max(440, ...values.map((pos) => pos.y + nodeHeight + 24));
    return { width: maxX, height: maxY };
  }, [positions]);

  const centerOf = (id: string) => {
    const pos = positions[id];
    if (!pos) return null;
    return { x: pos.x + nodeWidth / 2, y: pos.y + nodeHeight / 2 };
  };

  const hasDataFlow = (edgeId: string) => {
    const edge = edges.find((e) => e.id === edgeId);
    if (!edge) return false;
    const exec = executionStatus[edge.to] || executionStatus[edge.from];
    return exec?.status === "running";
  };

  const handlePointerDown = (event: PointerEvent<HTMLDivElement>, nodeId: string) => {
    const rect = event.currentTarget.getBoundingClientRect();
    event.currentTarget.setPointerCapture(event.pointerId);
    setDragging({ id: nodeId, offsetX: event.clientX - rect.left, offsetY: event.clientY - rect.top });
    setSelectedEdgeId(null);
  };

  const handlePointerMove = (event: PointerEvent<HTMLDivElement>) => {
    if (!dragging || !stageRef.current) return;
    const rect = stageRef.current.getBoundingClientRect();
    const maxX = Math.max(rect.width - nodeWidth - 16, 0);
    const maxY = Math.max(canvasSize.height - nodeHeight - 16, 0);
    setPositions((current) => ({
      ...current,
      [dragging.id]: {
        x: Math.max(16, Math.min(event.clientX - rect.left - dragging.offsetX, maxX)),
        y: Math.max(16, Math.min(event.clientY - rect.top - dragging.offsetY, maxY)),
      },
    }));
  };

  const handlePointerUp = () => {
    if (!dragging) return;
    setNodes((current) => [...current].sort((a, b) => {
      const posA = positions[a.id] ?? defaultPosition(0);
      const posB = positions[b.id] ?? defaultPosition(0);
      return posA.y === posB.y ? posA.x - posB.x : posA.y - posB.y;
    }));
    setDragging(null);
  };

  const deleteEdge = (edgeId: string) => { setEdges((current) => current.filter((edge) => edge.id !== edgeId)); setSelectedEdgeId(null); };

  return (
    <div>
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "16px" }}>
        <div style={{ display: "flex", alignItems: "center", gap: "12px" }}>
          <h2 style={{ color: "#e6edf3", margin: 0 }}>{t('console.topology.title')}</h2>
          <div style={{ display: "flex", gap: "6px" }}>
            {Object.values(executionStatus).some((e) => e.status === "running") && (
              <span style={{ color: "#3b82f6", fontSize: "11px", background: "#1a2332", padding: "2px 8px", borderRadius: "4px", display: "flex", alignItems: "center", gap: "4px" }}>
                <span style={{ width: "6px", height: "6px", borderRadius: "50%", background: "#3b82f6", animation: "pulse 1.5s infinite" }} />
                Running
              </span>
            )}
          </div>
        </div>
        {selectedEdgeId && (
          <button onClick={() => deleteEdge(selectedEdgeId)}
            style={{ border: "1px solid #f85149", background: "transparent", color: "#f85149", borderRadius: "6px", padding: "6px 10px", cursor: "pointer" }}>
            {t('console.topology.delete_link')}
          </button>
        )}
      </div>

      <div ref={stageRef} style={{ ...stageStyle, width: "100%" }}
        onPointerMove={handlePointerMove} onPointerUp={handlePointerUp} onPointerCancel={handlePointerUp}>
        {nodes.length === 0 && <p style={{ color: "#8b949e", padding: "16px", margin: 0 }}>{t('console.topology.no_components')}</p>}

        <svg width={canvasSize.width} height={canvasSize.height} viewBox={`0 0 ${canvasSize.width} ${canvasSize.height}`}
          style={{ position: "absolute", inset: 0, pointerEvents: "none" }}>
          <defs>
            <filter id="glow">
              <feGaussianBlur stdDeviation="3" result="coloredBlur" />
              <feMerge>
                <feMergeNode in="coloredBlur" />
                <feMergeNode in="SourceGraphic" />
              </feMerge>
            </filter>
          </defs>
          {edges.map((edge) => {
            const from = centerOf(edge.from);
            const to = centerOf(edge.to);
            if (!from || !to) return null;
            const midX = (from.x + to.x) / 2;
            const midY = (from.y + to.y) / 2;
            const selected = selectedEdgeId === edge.id;
            const isActive = hasDataFlow(edge.id);
            const edgeColor = isActive ? "#3b82f6" : selected ? "#f85149" : "#30363d";
            const edgeWidth = isActive ? 3 : selected ? 3 : 2;

            return (
              <g key={edge.id}>
                <line x1={from.x} y1={from.y} x2={to.x} y2={to.y} stroke="transparent" strokeWidth="14"
                  style={{ pointerEvents: "stroke", cursor: "pointer" }} onClick={() => setSelectedEdgeId(edge.id)} />
                <line x1={from.x} y1={from.y} x2={to.x} y2={to.y}
                  stroke={edgeColor} strokeWidth={edgeWidth}
                  strokeDasharray={isActive ? "0" : selected ? "0" : "6 6"}
                  filter={isActive ? "url(#glow)" : undefined} />
                {isActive && (
                  <circle r="4" fill="#3b82f6" filter="url(#glow)">
                    <animateMotion dur="1.5s" repeatCount="indefinite"
                      path={`M${from.x},${from.y} L${to.x},${to.y}`} />
                  </circle>
                )}
                {edge.label && (
                  <text x={midX} y={midY - 8} textAnchor="middle" fontSize="10" fill="#6b7280"
                    style={{ pointerEvents: "none" }}>
                    {edge.label}
                  </text>
                )}
                {selected && (
                  <g style={{ pointerEvents: "all", cursor: "pointer" }} onClick={() => deleteEdge(edge.id)}>
                    <circle cx={midX} cy={midY} r="11" fill="#f85149" />
                    <text x={midX} y={midY + 4} textAnchor="middle" fontSize="13" fill="#fff">x</text>
                  </g>
                )}
              </g>
            );
          })}
        </svg>

        {nodes.map((node) => {
          const pos = positions[node.id] ?? defaultPosition(0);
          const color = getNodeColor(node.node_type);
          const isDragging = dragging?.id === node.id;
          const exec = executionStatus[node.id];
          const statusColor = exec ? getStatusColor(exec.status) : getStatusColor(node.status);
          const isRunning = exec?.status === "running";
          const isFailed = exec?.status === "failed" || exec?.status === "error";
          const glow = isRunning ? getStatusGlow("running") : isFailed ? getStatusGlow("failed") : "none";

          return (
            <div key={node.id} style={{
              ...nodeStyle,
              left: pos.x,
              top: pos.y,
              borderLeft: `3px solid ${color}`,
              borderColor: isRunning ? "#3b82f6" : isFailed ? "#ef4444" : "#30363d",
              boxShadow: isDragging ? "0 12px 26px rgba(0,0,0,0.35)" : glow,
              cursor: isDragging ? "grabbing" : "grab",
              transition: "box-shadow 0.3s, border-color 0.3s",
            }}
              onPointerDown={(event) => handlePointerDown(event, node.id)}>
              <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
                <div style={{ color, fontSize: "11px", textTransform: "uppercase" }}>{node.node_type}</div>
                {isRunning && (
                  <div style={{ width: "8px", height: "8px", borderRadius: "50%", background: "#3b82f6", animation: "pulse 1.5s infinite" }} />
                )}
              </div>
              <div style={{ color: "#e6edf3", fontWeight: "bold", marginTop: "4px", whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis" }}>{node.name}</div>
              <div style={{ color: statusColor, fontSize: "12px", marginTop: "6px", display: "flex", alignItems: "center", gap: "4px" }}>
                <span style={{ width: "6px", height: "6px", borderRadius: "50%", background: statusColor, display: "inline-block",
                  animation: isRunning ? "pulse 1.5s infinite" : "none" }} />
                {exec?.status || node.status}
              </div>
            </div>
          );
        })}
      </div>

      <style>{`
        @keyframes pulse {
          0%, 100% { opacity: 1; }
          50% { opacity: 0.4; }
        }
      `}</style>

      <div style={{ color: "#8b949e", fontSize: "12px", marginTop: "8px" }}>
        {t('console.topology.hint')} {Object.keys(executionStatus).length > 0 && " — 节点状态实时更新"}
      </div>
    </div>
  );
}