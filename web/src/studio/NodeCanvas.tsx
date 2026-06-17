import { useState, useCallback, useRef, DragEvent, useEffect, useMemo } from "react";
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
  NodeChange,
  EdgeChange,
  useReactFlow,
} from "reactflow";
import "reactflow/dist/style.css";
import dagre from "dagre";
import { MobileView } from "./canvas/MobileView";
import { EditorPanel } from "./canvas/EditorPanel";
import { cloneSnapshot, downloadText, buildSnapshotSvg } from "./canvas/SnapshotHelper";
import type { CanvasSnapshot } from "./canvas/SnapshotHelper";
import { AgentDef, NodeData } from "./types";

const NODE_COLORS: Record<string, string> = {
  persona: "#7c3aed",
  tool: "#2563eb",
  knowledge: "#16a34a",
  skill: "#d97706",
  model: "#dc2626",
  agent: "#0891b2",
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

function getLayoutedElements(nodes: Node<NodeData>[], edges: Edge[], direction = "LR") {
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

function BaseNode({ data, selected }: NodeProps<NodeData>) {
  const color = NODE_COLORS[data.nodeType] || "#666";
  const label = data.label || NODE_LABELS[data.nodeType] || data.nodeType;

  const config: Record<string, { inputs: string[]; outputs: string[] }> = {
    persona: { inputs: ["system_prompt"], outputs: ["persona_config"] },
    tool: { inputs: ["params"], outputs: ["result"] },
    knowledge: { inputs: ["query"], outputs: ["data"] },
    skill: { inputs: ["input"], outputs: ["output"] },
    model: { inputs: ["prompt"], outputs: ["response"] },
    agent: {
      inputs: ["user_input", "persona_config", "tool_result", "knowledge_data", "skill_result", "model_response"],
      outputs: ["agent_output"],
    },
  };

  const ports = config[data.nodeType] || { inputs: [], outputs: [] };

  return (
    <div
      style={{
        background: "#1a1d23",
        border: `2px solid ${selected ? color : "#30363d"}`,
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
      {data.detail && <div style={{ color: "#8b949e", fontSize: "11px", marginTop: "4px" }}>{data.detail}</div>}

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

function makeInitialNodes(def: AgentDef): Node<NodeData>[] {
  if (def.tools.length > 0) {
    return [
      {
        id: "agent-node-1",
        type: "agent",
        position: { x: 400, y: 100 },
        data: { nodeType: "agent", label: "Agent", name: def.name || "My Agent", detail: def.persona || "assistant" },
      },
      ...def.tools.map((tool, i) => ({
        id: `tool-node-${i}`,
        type: "tool",
        position: { x: 50, y: 50 + i * 120 },
        data: { nodeType: "tool", label: "Tool", name: tool, detail: tool },
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
    ];
  }

  return [
    {
      id: "agent-node-1",
      type: "agent",
      position: { x: 350, y: 150 },
      data: { nodeType: "agent", label: "Agent", name: def.name || "My Agent", detail: def.persona || "assistant" },
    },
  ];
}

function CanvasInner({ def, onDefChange }: NodeCanvasProps) {
  const reactFlowWrapper = useRef<HTMLDivElement>(null);
  const reactFlow = useReactFlow<NodeData, Edge>();
  const [nodes, setNodes, reactFlowOnNodesChange] = useNodesState<NodeData>(makeInitialNodes(def));
  const [edges, setEdges, reactFlowOnEdgesChange] = useEdgesState([]);
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(nodes[0]?.id ?? null);
  const historyRef = useRef<{ past: CanvasSnapshot[]; future: CanvasSnapshot[] }>({ past: [], future: [] });
  const [historyCounts, setHistoryCounts] = useState({ undo: 0, redo: 0 });

  const selectedNode = useMemo(
    () => nodes.find((node) => node.id === selectedNodeId) ?? null,
    [nodes, selectedNodeId],
  );

  const syncHistoryCounts = useCallback(() => {
    setHistoryCounts({
      undo: historyRef.current.past.length,
      redo: historyRef.current.future.length,
    });
  }, []);

  const pushHistory = useCallback(
    (snapshot?: CanvasSnapshot) => {
      historyRef.current.past.push(snapshot ?? cloneSnapshot(nodes, edges));
      if (historyRef.current.past.length > 50) {
        historyRef.current.past.shift();
      }
      historyRef.current.future = [];
      syncHistoryCounts();
    },
    [edges, nodes, syncHistoryCounts],
  );

  const applySnapshot = useCallback(
    (snapshot: CanvasSnapshot) => {
      setNodes(snapshot.nodes);
      setEdges(snapshot.edges);
    },
    [setEdges, setNodes],
  );

  const undo = useCallback(() => {
    const previous = historyRef.current.past.pop();
    if (!previous) return;
    historyRef.current.future.push(cloneSnapshot(nodes, edges));
    applySnapshot(previous);
    syncHistoryCounts();
  }, [applySnapshot, edges, nodes, syncHistoryCounts]);

  const redo = useCallback(() => {
    const next = historyRef.current.future.pop();
    if (!next) return;
    historyRef.current.past.push(cloneSnapshot(nodes, edges));
    applySnapshot(next);
    syncHistoryCounts();
  }, [applySnapshot, edges, nodes, syncHistoryCounts]);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (!(event.ctrlKey || event.metaKey)) return;
      const key = event.key.toLowerCase();
      if (key === "z" && !event.shiftKey) {
        event.preventDefault();
        undo();
      }
      if ((key === "z" && event.shiftKey) || key === "y") {
        event.preventDefault();
        redo();
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [redo, undo]);

  const onNodesChange = useCallback(
    (changes: NodeChange[]) => {
      reactFlowOnNodesChange(changes);
    },
    [reactFlowOnNodesChange],
  );

  const onEdgesChange = useCallback(
    (changes: EdgeChange[]) => {
      reactFlowOnEdgesChange(changes);
    },
    [reactFlowOnEdgesChange],
  );

  const onConnect = useCallback(
    (params: Connection) => {
      pushHistory();
      setEdges((currentEdges) =>
        addEdge(
          {
            ...params,
            id: `edge-${params.source}-${params.sourceHandle}-${params.target}-${params.targetHandle}-${Date.now()}`,
            animated: true,
            style: { stroke: "#58a6ff", strokeWidth: 2 },
            markerEnd: { type: MarkerType.ArrowClosed, color: "#58a6ff" },
          },
          currentEdges,
        ),
      );
    },
    [pushHistory, setEdges],
  );

  const onDrop = useCallback(
    (event: DragEvent<HTMLDivElement>) => {
      event.preventDefault();
      const type = event.dataTransfer.getData("application/reactflow");
      if (!type || !reactFlowWrapper.current) return;

      const bounds = reactFlowWrapper.current.getBoundingClientRect();
      const position = reactFlow.screenToFlowPosition({
        x: event.clientX - bounds.left,
        y: event.clientY - bounds.top,
      });
      const newNode: Node<NodeData> = {
        id: getNodeId(),
        type,
        position,
        data: {
          nodeType: type,
          label: NODE_LABELS[type],
          name: `${NODE_LABELS[type]} ${nodes.length + 1}`,
          detail: "",
        },
      };

      pushHistory();
      setNodes((currentNodes) => [...currentNodes, newNode]);
      setSelectedNodeId(newNode.id);
    },
    [nodes.length, pushHistory, reactFlow, setNodes],
  );

  const onDragOver = useCallback((event: DragEvent<HTMLDivElement>) => {
    event.preventDefault();
    event.dataTransfer.dropEffect = "move";
  }, []);

  const onNodeClick = useCallback((_: React.MouseEvent, node: Node<NodeData>) => {
    setSelectedNodeId(node.id);
  }, []);

  const onPaneClick = useCallback(() => {
    setSelectedNodeId(null);
  }, []);

  const deleteSelected = useCallback(() => {
    if (!selectedNode) return;
    pushHistory();
    setNodes((currentNodes) => currentNodes.filter((node) => node.id !== selectedNode.id));
    setEdges((currentEdges) => currentEdges.filter((edge) => edge.source !== selectedNode.id && edge.target !== selectedNode.id));
    setSelectedNodeId(null);
  }, [pushHistory, selectedNode, setEdges, setNodes]);

  const updateSelectedNode = useCallback(
    (patch: Partial<NodeData>) => {
      if (!selectedNode) return;
      pushHistory();
      setNodes((currentNodes) =>
        currentNodes.map((node) =>
          node.id === selectedNode.id
            ? {
                ...node,
                data: { ...node.data, ...patch },
              }
            : node,
        ),
      );
    },
    [pushHistory, selectedNode, setNodes],
  );

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
    onDefChange(serializeToAgentDef());
  }, [onDefChange, serializeToAgentDef]);

  const onDragStart = (event: DragEvent<HTMLDivElement>, type: string) => {
    event.dataTransfer.setData("application/reactflow", type);
    event.dataTransfer.effectAllowed = "move";
  };

  const autoLayout = useCallback(() => {
    const { nodes: laidOut } = getLayoutedElements(nodes, edges);
    pushHistory();
    setNodes(laidOut);
  }, [edges, nodes, pushHistory, setNodes]);

  const zoomIn = useCallback(() => {
    reactFlow.zoomIn({ duration: 160 });
  }, [reactFlow]);

  const zoomOut = useCallback(() => {
    reactFlow.zoomOut({ duration: 160 });
  }, [reactFlow]);

  const takeSnapshot = useCallback(() => {
    const now = new Date().toISOString().replaceAll(":", "-");
    const snapshot = {
      created_at: new Date().toISOString(),
      def: serializeToAgentDef(),
      nodes: nodes.map((node) => ({
        id: node.id,
        type: node.data.nodeType,
        name: node.data.name,
        detail: node.data.detail,
        position: node.position,
        selected: node.id === selectedNodeId,
      })),
      edges: edges.map((edge) => ({
        id: edge.id,
        source: edge.source,
        sourceHandle: edge.sourceHandle,
        target: edge.target,
        targetHandle: edge.targetHandle,
      })),
    };
    downloadText(`studio-canvas-${now}.json`, JSON.stringify(snapshot, null, 2), "application/json");
    downloadText(`studio-canvas-${now}.svg`, buildSnapshotSvg(nodes, edges), "image/svg+xml");
  }, [edges, nodes, selectedNodeId, serializeToAgentDef]);

  const addNodeFromPalette = useCallback(
    (type: string) => {
      const newNode: Node<NodeData> = {
        id: getNodeId(),
        type,
        position: { x: 100, y: 80 + nodes.length * 90 },
        data: {
          nodeType: type,
          label: NODE_LABELS[type],
          name: `${NODE_LABELS[type]} ${nodes.length + 1}`,
          detail: "",
        },
      };
      pushHistory();
      setNodes((currentNodes) => [...currentNodes, newNode]);
      setSelectedNodeId(newNode.id);
    },
    [nodes.length, pushHistory, setNodes],
  );

  return (
    <MobileView
      nodes={nodes}
      selectedNodeId={selectedNodeId}
      historyCounts={historyCounts}
      paletteItems={PALETTE_CATEGORIES.flatMap((c) => c.items)}
      onAddNode={addNodeFromPalette}
      onUndo={undo}
      onRedo={redo}
      onSnapshot={takeSnapshot}
      onExport={exportDef}
      onSelectNode={setSelectedNodeId}
      onUpdateNode={updateSelectedNode}
      onDeleteNode={deleteSelected}
    >
      <div style={{ display: "flex", height: "600px", gap: "12px" }}>
        <div
          style={{
            width: "220px",
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
              <div style={{ color: "#8b949e", fontSize: "11px", marginBottom: "4px", textTransform: "uppercase" }}>{cat.name}</div>
              {cat.items.map((item) => (
                <div
                  key={item.type}
                  draggable
                  onDragStart={(event) => onDragStart(event, item.type)}
                  onDoubleClick={() => addNodeFromPalette(item.type)}
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
                  <span style={{ width: "10px", height: "10px", borderRadius: "50%", background: item.color, display: "inline-block" }} />
                  {item.label}
                </div>
              ))}
            </div>
          ))}
          <div style={{ marginTop: "16px", display: "flex", flexDirection: "column", gap: "6px" }}>
            <div style={{ display: "flex", gap: "4px" }}>
              <button onClick={undo} disabled={historyCounts.undo === 0} title="Ctrl+Z" style={{ fontSize: "12px", padding: "4px 8px", flex: 1, opacity: historyCounts.undo === 0 ? 0.5 : 1 }}>
                撤销
              </button>
              <button onClick={redo} disabled={historyCounts.redo === 0} title="Ctrl+Shift+Z" style={{ fontSize: "12px", padding: "4px 8px", flex: 1, opacity: historyCounts.redo === 0 ? 0.5 : 1 }}>
                重做
              </button>
            </div>
            <div style={{ display: "flex", gap: "4px" }}>
              <button onClick={autoLayout} title="自动排列" style={{ fontSize: "12px", padding: "4px 8px", flex: 1 }}>
                自动排列
              </button>
              <button onClick={zoomIn} title="放大" style={{ fontSize: "12px", padding: "4px 8px" }}>
                ＋
              </button>
              <button onClick={zoomOut} title="缩小" style={{ fontSize: "12px", padding: "4px 8px" }}>
                －
              </button>
            </div>
            <button onClick={takeSnapshot} style={{ fontSize: "12px", padding: "6px" }}>
              画布快照
            </button>
            <button onClick={exportDef} style={{ fontSize: "12px", padding: "6px" }}>
              导出为 AgentDef
            </button>
          </div>
          <EditorPanel
            selectedNode={selectedNode}
            onUpdate={updateSelectedNode}
            onDelete={deleteSelected}
          />
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
            onNodeDragStart={() => pushHistory()}
            nodeTypes={nodeTypes}
            fitView
            minZoom={0.2}
            maxZoom={3}
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
    </MobileView>
  );
}

export function NodeCanvas(props: NodeCanvasProps) {
  return (
    <ReactFlowProvider>
      <CanvasInner {...props} />
    </ReactFlowProvider>
  );
}