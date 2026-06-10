import { useState, useCallback, useRef, DragEvent } from "react";
import ReactFlow, {
  addEdge,
  Connection,
  Node,
  NodeTypes,
  useNodesState,
  useEdgesState,
  Controls,
  Background,
  MiniMap,
  ReactFlowProvider,
  Handle,
  Position,
  NodeProps,
  MarkerType,
  Edge,
} from "reactflow";
import "reactflow/dist/style.css";
import dagre from "dagre";

interface AgentDef {
  name: string;
  persona: string;
  model: string;
  tools: string[];
  knowledge: string[];
  skills: string[];
}

const NODE_COLORS: Record<string, string> = {
  persona: "#7c3aed",
  tool: "#3b82f6",
  knowledge: "#22c55e",
  skill: "#f59e0b",
  model: "#ef4444",
  agent: "#06b6d4",
};

const NODE_LABELS: Record<string, string> = {
  persona: "Persona",
  tool: "Tool",
  knowledge: "Knowledge",
  skill: "Skill",
  model: "Model",
  agent: "Agent",
};

const PALETTE_CATEGORIES = [
  {
    name: "基础组件",
    items: [
      { type: "agent", label: "Agent", color: NODE_COLORS.agent },
      { type: "persona", label: "Persona", color: NODE_COLORS.persona },
      { type: "model", label: "Model", color: NODE_COLORS.model },
    ],
  },
  {
    name: "功能组件",
    items: [
      { type: "tool", label: "Tool", color: NODE_COLORS.tool },
      { type: "knowledge", label: "Knowledge", color: NODE_COLORS.knowledge },
      { type: "skill", label: "Skill", color: NODE_COLORS.skill },
    ],
  },
];

function getLayoutedElements(nodes: Node[], edges: Edge[], direction = "LR") {
  const g = new dagre.graphlib.Graph();
  g.setDefaultEdgeLabel(() => ({}));
  g.setGraph({ rankdir: direction, nodesep: 60, ranksep: 100, marginx: 40, marginy: 40 });

  nodes.forEach((node) => {
    g.setNode(node.id, { width: 180, height: 80 });
  });
  edges.forEach((edge) => {
    g.setEdge(edge.source, edge.target);
  });
  dagre.layout(g);

  const laidOutNodes = nodes.map((node) => {
    const dagreNode = g.node(node.id);
    return {
      ...node,
      position: {
        x: dagreNode.x - 90,
        y: dagreNode.y - 40,
      },
    };
  });
  return { nodes: laidOutNodes, edges };
}

function BaseNode({ data, selected }: NodeProps) {
  const color = NODE_COLORS[data.nodeType] || "#666";
  const label = data.label || NODE_LABELS[data.nodeType] || data.nodeType;

  const config: Record<string, { inputs: string[]; outputs: string[] }> = {
    persona: { inputs: ["system_prompt"], outputs: ["persona_config"] },
    tool: { inputs: ["params"], outputs: ["result"] },
    knowledge: { inputs: ["query"], outputs: ["data"] },
    skill: { inputs: ["input"], outputs: ["output"] },
    model: { inputs: ["prompt"], outputs: ["response"] },
    agent: { inputs: ["user_input", "persona_config", "tool_result", "knowledge_data", "skill_result", "model_response"], outputs: ["agent_output"] },
  };

  const ports = config[data.nodeType] || { inputs: [], outputs: [] };

  return (
    <div
      style={{
        background: "#1a1d23",
        border: `2px solid ${selected ? color : "transparent"}`,
        borderRadius: "8px",
        padding: "10px 14px",
        minWidth: "160px",
        boxShadow: "0 4px 12px rgba(0,0,0,0.3)",
        position: "relative",
      }}
    >
      <div
        style={{
          background: color,
          color: "#fff",
          fontSize: "12px",
          fontWeight: 600,
          padding: "2px 8px",
          borderRadius: "4px",
          display: "inline-block",
          marginBottom: "6px",
        }}
      >
        {label}
      </div>
      <div style={{ color: "#e6edf3", fontSize: "13px", fontWeight: 500 }}>{data.name || "unnamed"}</div>
      {data.detail && (
        <div style={{ color: "#8b949e", fontSize: "11px", marginTop: "4px" }}>{data.detail}</div>
      )}

      {ports.inputs.map((port, i) => (
        <Handle
          key={`in-${port}`}
          type="target"
          position={Position.Left}
          id={port}
          style={{
            top: `${30 + i * 24}px`,
            background: color,
            border: `2px solid ${color}`,
            width: "10px",
            height: "10px",
          }}
          title={port}
        />
      ))}
      {ports.outputs.map((port, i) => (
        <Handle
          key={`out-${port}`}
          type="source"
          position={Position.Right}
          id={port}
          style={{
            top: `${30 + i * 24}px`,
            background: color,
            border: `2px solid ${color}`,
            width: "10px",
            height: "10px",
          }}
          title={port}
        />
      ))}
    </div>
  );
}

const nodeTypes: NodeTypes = {
  persona: BaseNode,
  tool: BaseNode,
  knowledge: BaseNode,
  skill: BaseNode,
  model: BaseNode,
  agent: BaseNode,
};

let nodeIdCounter = 0;
function getNodeId(): string {
  nodeIdCounter += 1;
  return `node-${nodeIdCounter}-${Date.now()}`;
}

interface NodeCanvasProps {
  def: AgentDef;
  onDefChange: (def: AgentDef) => void;
}

function CanvasInner({ def, onDefChange }: NodeCanvasProps) {
  const reactFlowWrapper = useRef<HTMLDivElement>(null);

  const initialNodes: Node[] = def.tools.length > 0
    ? [
        {
          id: "agent-node-1",
          type: "agent",
          position: { x: 400, y: 100 },
          data: { nodeType: "agent", label: "Agent", name: def.name || "My Agent", detail: def.persona },
        },
        ...def.tools.map((t, i) => ({
          id: `tool-node-${i}`,
          type: "tool",
          position: { x: 50, y: 50 + i * 120 },
          data: { nodeType: "tool", label: "Tool", name: t, detail: t },
        })),
        ...(def.model
          ? [
              {
                id: "model-node-1",
                type: "model",
                position: { x: 750, y: 100 },
                data: { nodeType: "model", label: "Model", name: def.model, detail: def.model },
              },
            ]
          : []),
      ]
    : [
        {
          id: "agent-node-1",
          type: "agent",
          position: { x: 350, y: 150 },
          data: { nodeType: "agent", label: "Agent", name: "My Agent", detail: "assistant" },
        },
      ];

  const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState([]);
  const [selectedNode, setSelectedNode] = useState<Node | null>(null);
  const [history, setHistory] = useState<{ nodes: Node[]; edges: Edge[] }[]>([]);
  const [historyIndex, setHistoryIndex] = useState(-1);

  const pushHistory = useCallback((nds: Node[], eds: Edge[]) => {
    setHistory((h) => {
      const newHistory = h.slice(0, historyIndex + 1);
      newHistory.push({ nodes: JSON.parse(JSON.stringify(nds)), edges: JSON.parse(JSON.stringify(eds)) });
      if (newHistory.length > 50) newHistory.shift();
      return newHistory;
    });
    setHistoryIndex((i) => Math.min(i + 1, 49));
  }, [historyIndex]);

  const undo = useCallback(() => {
    if (historyIndex < 0) return;
    const prev = history[historyIndex];
    if (prev) {
      setNodes(prev.nodes);
      setEdges(prev.edges);
      setHistoryIndex((i) => i - 1);
    }
  }, [history, historyIndex, setNodes, setEdges]);

  const redo = useCallback(() => {
    if (historyIndex + 1 >= history.length) return;
    const next = history[historyIndex + 2];
    if (next) {
      setNodes(next.nodes);
      setEdges(next.edges);
      setHistoryIndex((i) => i + 1);
    }
  }, [history, historyIndex, setNodes, setEdges]);

  const onConnect = useCallback(
    (params: Connection) => {
      setEdges((eds) => {
        const newEdge = {
          ...params,
          animated: true,
          style: { stroke: "#58a6ff", strokeWidth: 2 },
          markerEnd: { type: MarkerType.ArrowClosed, color: "#58a6ff" },
        };
        return addEdge(newEdge, eds);
      });
    },
    [setEdges],
  );

  const onDrop = useCallback(
    (event: DragEvent<HTMLDivElement>) => {
      event.preventDefault();
      const type = event.dataTransfer.getData("application/reactflow");
      if (!type) return;
      const position = reactFlowWrapper.current
        ? {
            x: event.clientX - reactFlowWrapper.current.getBoundingClientRect().left - 80,
            y: event.clientY - reactFlowWrapper.current.getBoundingClientRect().top - 20,
          }
        : { x: 100, y: 100 };
      const newNode: Node = {
        id: getNodeId(),
        type,
        position,
        data: { nodeType: type, label: NODE_LABELS[type], name: `${NODE_LABELS[type]} ${nodes.length + 1}`, detail: "" },
      };
      setNodes((nds) => {
        pushHistory(nds, edges);
        return [...nds, newNode];
      });
    },
    [nodes, setNodes, pushHistory, edges],
  );

  const onDragOver = useCallback((event: DragEvent<HTMLDivElement>) => {
    event.preventDefault();
    event.dataTransfer.dropEffect = "move";
  }, []);

  const onNodeClick = useCallback((_: React.MouseEvent, node: Node) => {
    setSelectedNode(node);
  }, []);

  const onPaneClick = useCallback(() => {
    setSelectedNode(null);
  }, []);

  const deleteSelected = useCallback(() => {
    if (selectedNode) {
      pushHistory(nodes, edges);
      setNodes((nds) => nds.filter((n) => n.id !== selectedNode.id));
      setEdges((eds) => eds.filter((e) => e.source !== selectedNode.id && e.target !== selectedNode.id));
      setSelectedNode(null);
    }
  }, [selectedNode, setNodes, setEdges, pushHistory, nodes, edges]);

  const serializeToAgentDef = useCallback((): AgentDef => {
    const result: AgentDef = { name: "", persona: "assistant", model: "deepseek-chat", tools: [], knowledge: [], skills: [] };
    for (const node of nodes) {
      const d = node.data;
      switch (d.nodeType) {
        case "agent":
          result.name = d.name || result.name;
          result.persona = d.detail || result.persona;
          break;
        case "tool":
          result.tools.push(d.name);
          break;
        case "knowledge":
          result.knowledge.push(d.name);
          break;
        case "skill":
          result.skills.push(d.name);
          break;
        case "model":
          result.model = d.name || result.model;
          break;
        case "persona":
          result.persona = d.detail || d.name || result.persona;
          break;
      }
    }
    return result;
  }, [nodes]);

  const exportDef = useCallback(() => {
    const agentDef = serializeToAgentDef();
    onDefChange(agentDef);
  }, [serializeToAgentDef, onDefChange]);

  const onDragStart = (event: DragEvent<HTMLDivElement>, type: string) => {
    event.dataTransfer.setData("application/reactflow", type);
    event.dataTransfer.effectAllowed = "move";
  };

  const autoLayout = useCallback(() => {
    const { nodes: laidOut } = getLayoutedElements(nodes, edges);
    pushHistory(nodes, edges);
    setNodes(laidOut);
  }, [nodes, edges, setNodes, pushHistory]);

  const zoomIn = useCallback(() => {
    const viewport = document.querySelector(".react-flow__viewport");
    if (viewport) {
      const transform = viewport.getAttribute("transform");
      if (transform) {
        const match = transform.match(/scale\(([\d.]+)\)/);
        const scale = match ? parseFloat(match[1]) * 1.2 : 1.2;
        viewport.setAttribute("transform", `translate(0,0) scale(${Math.min(scale, 3)})`);
      }
    }
  }, []);

  const zoomOut = useCallback(() => {
    const viewport = document.querySelector(".react-flow__viewport");
    if (viewport) {
      const transform = viewport.getAttribute("transform");
      if (transform) {
        const match = transform.match(/scale\(([\d.]+)\)/);
        const scale = match ? parseFloat(match[1]) / 1.2 : 0.8;
        viewport.setAttribute("transform", `translate(0,0) scale(${Math.max(scale, 0.2)})`);
      }
    }
  }, []);

  return (
    <div style={{ display: "flex", height: "600px", gap: "12px" }}>
      <div
        style={{
          width: "200px",
          background: "#161b22",
          borderRadius: "8px",
          border: "1px solid #30363d",
          padding: "12px",
          flexShrink: 0,
          overflowY: "auto",
        }}
      >
        <h4 style={{ color: "#e6edf3", margin: "0 0 12px 0", fontSize: "14px" }}>组件库</h4>
        {PALETTE_CATEGORIES.map((cat) => (
          <div key={cat.name} style={{ marginBottom: "12px" }}>
            <div style={{ color: "#8b949e", fontSize: "11px", marginBottom: "4px", textTransform: "uppercase", letterSpacing: "0.5px" }}>
              {cat.name}
            </div>
            {cat.items.map((item) => (
              <div
                key={item.type}
                draggable
                onDragStart={(e) => onDragStart(e, item.type)}
                style={{
                  padding: "8px 10px",
                  marginBottom: "4px",
                  borderRadius: "6px",
                  background: "#1a1d23",
                  border: `1px solid ${item.color}44`,
                  cursor: "grab",
                  display: "flex",
                  alignItems: "center",
                  gap: "8px",
                  color: "#e6edf3",
                  fontSize: "13px",
                }}
              >
                <span
                  style={{
                    width: "10px",
                    height: "10px",
                    borderRadius: "50%",
                    background: item.color,
                    display: "inline-block",
                  }}
                />
                {item.label}
              </div>
            ))}
          </div>
        ))}
        <div style={{ marginTop: "16px", display: "flex", flexDirection: "column", gap: "6px" }}>
          <div style={{ display: "flex", gap: "4px" }}>
            <button onClick={undo} disabled={historyIndex < 0} title="撤销" style={{ fontSize: "12px", padding: "4px 8px", flex: 1, opacity: historyIndex < 0 ? 0.5 : 1 }}>
              ↩ 撤销
            </button>
            <button onClick={redo} disabled={historyIndex + 1 >= history.length} title="重做" style={{ fontSize: "12px", padding: "4px 8px", flex: 1, opacity: historyIndex + 1 >= history.length ? 0.5 : 1 }}>
              ↪ 重做
            </button>
          </div>
          <div style={{ display: "flex", gap: "4px" }}>
            <button onClick={autoLayout} title="自动排列" style={{ fontSize: "12px", padding: "4px 8px", flex: 1 }}>
              ⊞ 自动排列
            </button>
            <button onClick={zoomIn} title="放大" style={{ fontSize: "12px", padding: "4px 8px" }}>＋</button>
            <button onClick={zoomOut} title="缩小" style={{ fontSize: "12px", padding: "4px 8px" }}>－</button>
          </div>
          <button onClick={exportDef} style={{ fontSize: "12px", padding: "6px" }}>
            导出为 AgentDef
          </button>
          {selectedNode && (
            <button onClick={deleteSelected} style={{ fontSize: "12px", padding: "6px", background: "#ef4444" }}>
              删除节点
            </button>
          )}
        </div>
      </div>
      <div
        ref={reactFlowWrapper}
        style={{ flex: 1, borderRadius: "8px", overflow: "hidden", border: "1px solid #30363d" }}
        onDrop={onDrop}
        onDragOver={onDragOver}
      >
        <ReactFlow
          nodes={nodes}
          edges={edges}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          onConnect={onConnect}
          onNodeClick={onNodeClick}
          onPaneClick={onPaneClick}
          nodeTypes={nodeTypes}
          fitView
          style={{ background: "#0d1117" }}
          deleteKeyCode={["Backspace", "Delete"]}
        >
          <Controls style={{ background: "#161b22", border: "1px solid #30363d", borderRadius: "6px" }} />
          <Background color="#30363d" gap={20} />
          <MiniMap
            style={{ background: "#161b22", border: "1px solid #30363d", borderRadius: "6px" }}
            nodeColor={(node) => NODE_COLORS[node.data?.nodeType] || "#666"}
          />
        </ReactFlow>
      </div>
    </div>
  );
}

export function NodeCanvas(props: NodeCanvasProps) {
  return (
    <ReactFlowProvider>
      <CanvasInner {...props} />
    </ReactFlowProvider>
  );
}