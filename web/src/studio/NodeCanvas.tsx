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
  EdgeLabelRenderer,
  getBezierPath,
} from "reactflow";
import "reactflow/dist/style.css";
import dagre from "dagre";
import { MobileView } from "./canvas/MobileView";
import EditorPanel from "./canvas/EditorPanel";
import { cloneSnapshot, downloadText, buildSnapshotSvg } from "./canvas/SnapshotHelper";
import type { CanvasSnapshot } from "./canvas/SnapshotHelper";
import { AgentDef, NodeData } from "./types";
import { useTranslation } from '../i18n';

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

const NODE_PORT_CONFIG: Record<string, { inputs: string[]; outputs: string[] }> = {
  persona: { inputs: ["system_prompt"], outputs: ["persona_config"] },
  tool: { inputs: ["params"], outputs: ["result"] },
  knowledge: { inputs: ["query"], outputs: ["data"] },
  skill: { inputs: ["input"], outputs: ["output"] },
  model: { inputs: ["prompt"], outputs: ["response"] },
  agent: { inputs: ["user_input", "persona_config", "tool_result", "knowledge_data", "skill_result", "model_response"], outputs: ["agent_output"] },
};

const PORT_TYPES: Record<string, string> = {
  system_prompt: "string",
  persona_config: "PersonaConfig",
  params: "Params",
  result: "any",
  query: "string",
  data: "any",
  input: "any",
  output: "any",
  prompt: "string",
  response: "string",
  user_input: "string",
  tool_result: "any",
  knowledge_data: "any",
  skill_result: "any",
  model_response: "string",
  agent_output: "string",
};

const PALETTE_CATEGORIES = [
  {
    name: "basic_components",
    items: [
      { type: "agent", label: "Agent", color: NODE_COLORS.agent },
      { type: "persona", label: "Persona", color: NODE_COLORS.persona },
      { type: "model", label: "Model", color: NODE_COLORS.model },
    ],
  },
  {
    name: "functional_components",
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
  nodes.forEach((node) => { g.setNode(node.id, { width: 180, height: 80 }); });
  edges.forEach((edge) => { g.setEdge(edge.source, edge.target); });
  dagre.layout(g);
  const laidOutNodes = nodes.map((node) => {
    const dagreNode = g.node(node.id);
    return { ...node, position: { x: dagreNode.x - 90, y: dagreNode.y - 40 } };
  });
  return { nodes: laidOutNodes, edges };
}

function LabeledEdge({ id, sourceHandle, selected }: Edge & { selected?: boolean }) {
  const edgePathParams = { sourceX: 0, sourceY: 0, targetX: 0, targetY: 0, sourcePosition: Position.Right, targetPosition: Position.Left };
  const [edgePath, labelX, labelY] = getBezierPath(edgePathParams);
  const sourcePortLabel = sourceHandle || "";
  const sourceType = PORT_TYPES[sourcePortLabel] || "any";

  return (
    <>
      <path id={id} className="react-flow__edge-path"
        d={edgePath}
        style={{
          stroke: selected ? "#f85149" : "#58a6ff",
          strokeWidth: selected ? 3 : 2,
          strokeDasharray: selected ? "0" : "0",
        }}
      />
      <EdgeLabelRenderer>
        <div
          style={{
            position: "absolute",
            transform: `translate(-50%, -50%) translate(${labelX}px,${labelY}px)`,
            background: selected ? "#f85149" : "#1a1d23",
            border: `1px solid ${selected ? "#f85149" : "#30363d"}`,
            borderRadius: "4px",
            padding: "2px 8px",
            fontSize: "10px",
            color: "#e6edf3",
            pointerEvents: "none",
            whiteSpace: "nowrap",
            zIndex: 10,
          }}
        >
          <span style={{ color: "#8b949e" }}>{sourcePortLabel} </span>
          <span style={{ color: "#58a6ff" }}>{sourceType}</span>
        </div>
      </EdgeLabelRenderer>
    </>
  );
}

const edgeTypes = { labeled: LabeledEdge };

function BaseNode({ data, selected }: NodeProps<NodeData>) {
  const color = NODE_COLORS[data.nodeType] || "#666";
  const label = data.label || NODE_LABELS[data.nodeType] || data.nodeType;
  const [hovered, setHovered] = useState(false);
  const ports = NODE_PORT_CONFIG[data.nodeType] || { inputs: [], outputs: [] };

  return (
    <div
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{ position: "relative" }}
    >
      <div
        style={{
          background: "#1a1d23",
          border: `2px solid ${selected || data.expanded ? color : "#30363d"}`,
          borderRadius: "8px",
          padding: "10px 14px",
          minWidth: "160px",
          boxShadow: hovered ? `0 0 0 2px ${color}44, 0 4px 12px rgba(0,0,0,0.3)` : "0 4px 12px rgba(0,0,0,0.3)",
          position: "relative",
          transition: "box-shadow 0.15s, border-color 0.15s",
        }}
      >
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "6px" }}>
          <div
            style={{
              background: color,
              color: "#fff",
              fontSize: "12px",
              fontWeight: 600,
              padding: "2px 8px",
              borderRadius: "4px",
              display: "inline-block",
            }}
          >
            {label}
          </div>
          {data.expanded !== undefined && (
            <span style={{ color: "#8b949e", fontSize: "10px" }}>
              {data.expanded ? "▼" : "▶"}
            </span>
          )}
        </div>
        <div style={{ color: "#e6edf3", fontSize: "13px", fontWeight: 500 }}>{data.name || "unnamed"}</div>
        {data.detail && data.expanded && (
          <div style={{ color: "#8b949e", fontSize: "11px", marginTop: "4px", wordBreak: "break-word" }}>{data.detail}</div>
        )}

        {ports.inputs.map((port, i) => (
          <Handle
            key={`in-${port}`}
            type="target"
            position={Position.Left}
            id={port}
            style={{
              top: `${28 + i * 24}px`,
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
              top: `${28 + i * 24}px`,
              background: color,
              border: `2px solid ${color}`,
              width: "10px",
              height: "10px",
            }}
            title={port}
          />
        ))}
      </div>

      {hovered && (
        <div
          style={{
            position: "absolute",
            top: "100%",
            left: "50%",
            transform: "translateX(-50%)",
            marginTop: "8px",
            background: "#0d1117",
            border: "1px solid #30363d",
            borderRadius: "6px",
            padding: "8px 10px",
            zIndex: 100,
            minWidth: "180px",
            boxShadow: "0 8px 24px rgba(0,0,0,0.4)",
            pointerEvents: "none",
          }}
        >
          <div style={{ display: "flex", alignItems: "center", gap: "6px", marginBottom: "6px" }}>
            <span style={{ width: "8px", height: "8px", borderRadius: "50%", background: color }} />
            <span style={{ color: "#e6edf3", fontSize: "12px", fontWeight: 600 }}>{label}</span>
          </div>
          <div style={{ color: "#8b949e", fontSize: "11px", marginBottom: "4px" }}>{data.name}</div>
          {data.detail && <div style={{ color: "#8b949e", fontSize: "10px", marginBottom: "4px" }}>{data.detail}</div>}
          <div style={{ color: "#6b7280", fontSize: "10px" }}>
            Ports: in({ports.inputs.join(", ")}) out({ports.outputs.join(", ")})
          </div>
        </div>
      )}
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
  const { t } = useTranslation();
  const reactFlowWrapper = useRef<HTMLDivElement>(null);
  const reactFlow = useReactFlow<NodeData, Edge>();
  const [nodes, setNodes, reactFlowOnNodesChange] = useNodesState<NodeData>(makeInitialNodes(def));
  const [edges, setEdges, reactFlowOnEdgesChange] = useEdgesState([]);
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(nodes[0]?.id ?? null);
  const [selectedEdgeId, setSelectedEdgeId] = useState<string | null>(null);
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
      if (event.key === "Delete" || event.key === "Backspace") {
        if (selectedEdgeId && !(event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement)) {
          event.preventDefault();
          setEdges((current) => current.filter((e) => e.id !== selectedEdgeId));
          setSelectedEdgeId(null);
          return;
        }
      }
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
  }, [redo, undo, selectedEdgeId, setEdges]);

  const onNodesChange = useCallback(
    (changes: NodeChange[]) => {
      reactFlowOnNodesChange(changes);
    },
    [reactFlowOnNodesChange],
  );

  const onEdgesChange = useCallback(
    (changes: EdgeChange[]) => {
      reactFlowOnEdgesChange(changes);
      for (const change of changes) {
        if (change.type === "remove") {
          setSelectedEdgeId((prev) => (prev === change.id ? null : prev));
        }
      }
    },
    [reactFlowOnEdgesChange],
  );

  const isValidConnection = useCallback(
    (connection: Connection) => {
      if (connection.source === connection.target) return false;
      const existing = edges.find(
        (e) => e.source === connection.source && e.sourceHandle === connection.sourceHandle && e.target === connection.target && e.targetHandle === connection.targetHandle,
      );
      return !existing;
    },
    [edges],
  );

  const onConnect = useCallback(
    (params: Connection) => {
      if (!isValidConnection(params)) return;
      pushHistory();
      const edgeType = "labeled";
      setEdges((currentEdges) =>
        addEdge(
          {
            ...params,
            id: `edge-${params.source}-${params.sourceHandle}-${params.target}-${params.targetHandle}-${Date.now()}`,
            type: edgeType,
            animated: true,
            style: { stroke: "#58a6ff", strokeWidth: 2 },
            markerEnd: { type: MarkerType.ArrowClosed, color: "#58a6ff" },
          },
          currentEdges,
        ),
      );
    },
    [isValidConnection, pushHistory, setEdges],
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
    setSelectedNodeId((prev) => (prev === node.id ? prev : node.id));
    setSelectedEdgeId(null);
  }, []);

  const onNodeDoubleClick = useCallback(
    (_: React.MouseEvent, node: Node<NodeData>) => {
      setNodes((currentNodes) =>
        currentNodes.map((n) =>
          n.id === node.id ? { ...n, data: { ...n.data, expanded: !n.data.expanded } } : n,
        ),
      );
    },
    [setNodes],
  );

  const onPaneClick = useCallback(() => {
    setSelectedNodeId(null);
    setSelectedEdgeId(null);
  }, []);

  const onEdgeClick = useCallback((_: React.MouseEvent, edge: Edge) => {
    setSelectedEdgeId((prev) => (prev === edge.id ? null : edge.id));
    setSelectedNodeId(null);
  }, []);

  const deleteSelected = useCallback(() => {
    if (selectedEdgeId) {
      setEdges((currentEdges) => currentEdges.filter((edge) => edge.id !== selectedEdgeId));
      setSelectedEdgeId(null);
      return;
    }
    if (!selectedNode) return;
    pushHistory();
    setNodes((currentNodes) => currentNodes.filter((node) => node.id !== selectedNode.id));
    setEdges((currentEdges) => currentEdges.filter((edge) => edge.source !== selectedNode.id && edge.target !== selectedNode.id));
    setSelectedNodeId(null);
  }, [pushHistory, selectedEdgeId, selectedNode, setEdges, setNodes]);

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

  const edgesWithSelection = edges.map((e) => ({ ...e, selected: e.id === selectedEdgeId }));

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
          <h4 style={{ color: "#e6edf3", margin: "0 0 12px 0", fontSize: "14px" }}>{t('studio.canvas.component_lib')}</h4>
          {PALETTE_CATEGORIES.map((cat) => (
            <div key={cat.name} style={{ marginBottom: "12px" }}>
              <div style={{ color: "#8b949e", fontSize: "11px", marginBottom: "4px", textTransform: "uppercase" }}>{t(`node_canvas.${cat.name}`)}</div>
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
                {t('node_canvas.undo')}
              </button>
              <button onClick={redo} disabled={historyCounts.redo === 0} title="Ctrl+Shift+Z" style={{ fontSize: "12px", padding: "4px 8px", flex: 1, opacity: historyCounts.redo === 0 ? 0.5 : 1 }}>
                {t('node_canvas.redo')}
              </button>
            </div>
            <div style={{ display: "flex", gap: "4px" }}>
              <button onClick={autoLayout} title={t('node_canvas.auto_layout')} style={{ fontSize: "12px", padding: "4px 8px", flex: 1 }}>
                {t('node_canvas.auto_layout')}
              </button>
              <button onClick={zoomIn} title={t('node_canvas.zoom_in')} style={{ fontSize: "12px", padding: "4px 8px" }}>
                ＋
              </button>
              <button onClick={zoomOut} title={t('node_canvas.zoom_out')} style={{ fontSize: "12px", padding: "4px 8px" }}>
                －
              </button>
            </div>
            <button onClick={takeSnapshot} style={{ fontSize: "12px", padding: "6px" }}>
              {t('node_canvas.snapshot')}
            </button>
            <button onClick={exportDef} style={{ fontSize: "12px", padding: "6px" }}>
              {t('node_canvas.export_def')}
            </button>
          </div>
          <EditorPanel
            selectedNode={selectedNode}
            selectedEdgeId={selectedEdgeId}
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
            edges={edgesWithSelection}
            onNodesChange={onNodesChange}
            onEdgesChange={onEdgesChange}
            onConnect={onConnect}
            onNodeClick={onNodeClick}
            onNodeDoubleClick={onNodeDoubleClick}
            onEdgeClick={onEdgeClick}
            onPaneClick={onPaneClick}
            onNodeDragStart={() => pushHistory()}
            isValidConnection={isValidConnection}
            nodeTypes={nodeTypes}
            edgeTypes={edgeTypes}
            defaultEdgeOptions={{ type: "labeled" }}
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