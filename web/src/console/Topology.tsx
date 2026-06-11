import { useEffect, useMemo, useRef, useState, type CSSProperties, type PointerEvent } from "react";
import { api } from "../api";

interface TopologyNode {
  id: string;
  name: string;
  node_type: string;
  status: string;
}

interface TopologyEdge {
  id: string;
  from: string;
  to: string;
}

interface NodePosition {
  x: number;
  y: number;
}

const nodeWidth = 180;
const nodeHeight = 82;

const stageStyle: CSSProperties = {
  position: "relative",
  minHeight: "440px",
  border: "1px solid #30363d",
  borderRadius: "8px",
  background: "#0d1117",
  overflow: "hidden",
};

const nodeStyle: CSSProperties = {
  position: "absolute",
  width: `${nodeWidth}px`,
  height: `${nodeHeight}px`,
  background: "#161b22",
  border: "1px solid #30363d",
  borderRadius: "8px",
  padding: "12px 14px",
  cursor: "grab",
  userSelect: "none",
  boxSizing: "border-box",
};

function defaultPosition(index: number): NodePosition {
  const columns = 3;
  return {
    x: 24 + (index % columns) * 230,
    y: 24 + Math.floor(index / columns) * 132,
  };
}

export default function Topology() {
  const [nodes, setNodes] = useState<TopologyNode[]>([]);
  const [positions, setPositions] = useState<Record<string, NodePosition>>({});
  const [edges, setEdges] = useState<TopologyEdge[]>([]);
  const [dragging, setDragging] = useState<{ id: string; offsetX: number; offsetY: number } | null>(null);
  const [selectedEdgeId, setSelectedEdgeId] = useState<string | null>(null);
  const stageRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    api.getComponentTopology()
      .then((result: TopologyNode[]) => {
        setNodes(result);
        setPositions((current) => {
          const next = { ...current };
          result.forEach((node, index) => {
            if (!next[node.id]) {
              next[node.id] = defaultPosition(index);
            }
          });
          return next;
        });
        setEdges((current) => {
          if (current.length > 0 || result.length < 2) {
            return current;
          }
          return result.slice(1).map((node, index) => ({
            id: `${result[index].id}->${node.id}`,
            from: result[index].id,
            to: node.id,
          }));
        });
      })
      .catch(() => {});
  }, []);

  const canvasSize = useMemo(() => {
    const values = Object.values(positions);
    const maxX = Math.max(760, ...values.map((pos) => pos.x + nodeWidth + 24));
    const maxY = Math.max(440, ...values.map((pos) => pos.y + nodeHeight + 24));
    return { width: maxX, height: maxY };
  }, [positions]);

  const getNodeColor = (type: string) => {
    switch (type) {
      case "agent": return "#58a6ff";
      case "tool": return "#3fb950";
      case "model": return "#d29922";
      case "knowledge": return "#bc8cff";
      case "channel": return "#f0883e";
      case "capability": return "#79c0ff";
      default: return "#8b949e";
    }
  };

  const centerOf = (id: string) => {
    const pos = positions[id];
    if (!pos) {
      return null;
    }
    return { x: pos.x + nodeWidth / 2, y: pos.y + nodeHeight / 2 };
  };

  const handlePointerDown = (event: PointerEvent<HTMLDivElement>, nodeId: string) => {
    const rect = event.currentTarget.getBoundingClientRect();
    event.currentTarget.setPointerCapture(event.pointerId);
    setDragging({
      id: nodeId,
      offsetX: event.clientX - rect.left,
      offsetY: event.clientY - rect.top,
    });
    setSelectedEdgeId(null);
  };

  const handlePointerMove = (event: PointerEvent<HTMLDivElement>) => {
    if (!dragging || !stageRef.current) {
      return;
    }
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
    if (!dragging) {
      return;
    }
    setNodes((current) =>
      [...current].sort((a, b) => {
        const posA = positions[a.id] ?? defaultPosition(0);
        const posB = positions[b.id] ?? defaultPosition(0);
        return posA.y === posB.y ? posA.x - posB.x : posA.y - posB.y;
      }),
    );
    setDragging(null);
  };

  const deleteEdge = (edgeId: string) => {
    setEdges((current) => current.filter((edge) => edge.id !== edgeId));
    setSelectedEdgeId(null);
  };

  return (
    <div>
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "16px" }}>
        <h2 style={{ color: "#e6edf3", margin: 0 }}>Component Topology</h2>
        {selectedEdgeId && (
          <button
            onClick={() => deleteEdge(selectedEdgeId)}
            style={{ border: "1px solid #f85149", background: "transparent", color: "#f85149", borderRadius: "6px", padding: "6px 10px", cursor: "pointer" }}
          >
            Delete link
          </button>
        )}
      </div>
      <div
        ref={stageRef}
        style={{ ...stageStyle, width: "100%" }}
        onPointerMove={handlePointerMove}
        onPointerUp={handlePointerUp}
        onPointerCancel={handlePointerUp}
      >
        {nodes.length === 0 && (
          <p style={{ color: "#8b949e", padding: "16px", margin: 0 }}>No components registered yet.</p>
        )}
        <svg
          width={canvasSize.width}
          height={canvasSize.height}
          viewBox={`0 0 ${canvasSize.width} ${canvasSize.height}`}
          style={{ position: "absolute", inset: 0, pointerEvents: "none" }}
        >
          {edges.map((edge) => {
            const from = centerOf(edge.from);
            const to = centerOf(edge.to);
            if (!from || !to) {
              return null;
            }
            const midX = (from.x + to.x) / 2;
            const midY = (from.y + to.y) / 2;
            const selected = selectedEdgeId === edge.id;
            return (
              <g key={edge.id}>
                <line
                  x1={from.x}
                  y1={from.y}
                  x2={to.x}
                  y2={to.y}
                  stroke="transparent"
                  strokeWidth="14"
                  style={{ pointerEvents: "stroke", cursor: "pointer" }}
                  onClick={() => setSelectedEdgeId(edge.id)}
                />
                <line
                  x1={from.x}
                  y1={from.y}
                  x2={to.x}
                  y2={to.y}
                  stroke={selected ? "#f85149" : "#30363d"}
                  strokeWidth={selected ? "3" : "2"}
                  strokeDasharray={selected ? "0" : "6 6"}
                />
                {selected && (
                  <g
                    style={{ pointerEvents: "all", cursor: "pointer" }}
                    onClick={() => deleteEdge(edge.id)}
                  >
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
          return (
            <div
              key={node.id}
              style={{
                ...nodeStyle,
                left: pos.x,
                top: pos.y,
                borderLeft: `3px solid ${color}`,
                boxShadow: isDragging ? "0 12px 26px rgba(0,0,0,0.35)" : "none",
                cursor: isDragging ? "grabbing" : "grab",
              }}
              onPointerDown={(event) => handlePointerDown(event, node.id)}
            >
              <div style={{ color, fontSize: "11px", textTransform: "uppercase" }}>
                {node.node_type}
              </div>
              <div style={{ color: "#e6edf3", fontWeight: "bold", marginTop: "4px", whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis" }}>
                {node.name}
              </div>
              <div style={{ color: "#3fb950", fontSize: "12px", marginTop: "6px" }}>● {node.status}</div>
            </div>
          );
        })}
      </div>
      <div style={{ color: "#8b949e", fontSize: "12px", marginTop: "8px" }}>
        Drag nodes to rearrange. Select a link, then delete it from the canvas.
      </div>
    </div>
  );
}
