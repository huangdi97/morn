import { useState, useCallback, useRef, useEffect } from "react";
import ReactFlow, {
  addEdge,
  Connection,
  Node,
  Edge,
  NodeProps,
  useNodesState,
  useEdgesState,
  useReactFlow,
  Controls,
  Background,
  MiniMap,
  ReactFlowProvider,
  MarkerType,
  NodeChange,
  EdgeChange,
  Handle,
  Position,
} from "reactflow";
import "reactflow/dist/style.css";
import dagre from "dagre";
import { api } from "../api";

interface WorkflowStep {
  id: string;
  action: { type: string; config: Record<string, unknown> };
  depends_on: string[];
  timeout_secs: number;
}

interface WorkflowTemplate {
  id: string;
  name: string;
  description: string;
  steps: WorkflowStep[];
}

interface WorkflowNodeData {
  stepName: string;
  actionType: string;
  status?: string;
}

const ACTION_COLORS: Record<string, string> = {
  llm_call: "#7c3aed",
  tool_exec: "#2563eb",
  api_request: "#16a34a",
  conditional: "#d97706",
  loop: "#dc2626",
  transform: "#0891b2",
  notify: "#db2777",
  timer: "#f59e0b",
  webhook: "#ec4899",
  agent_call: "#8b5cf6",
  analyze: "#06b6d4",
  web_search: "#3b82f6",
  kb_query: "#10b981",
  read_file: "#6366f1",
  generate_report: "#f97316",
  write_file: "#22c55e",
  default: "#666",
};

const STATUS_COLORS: Record<string, string> = {
  success: "#3fb950",
  failed: "#f85149",
  running: "#58a6ff",
  pending: "#8b949e",
  skipped: "#6b7280",
};

const MOCK_TEMPLATES: WorkflowTemplate[] = [
  {
    id: "research-pipeline",
    name: "Research Pipeline",
    description: "Search → Extract → Summarize → Report",
    steps: [
      { id: "search", action: { type: "api_request", config: { endpoint: "search" } }, depends_on: [], timeout_secs: 30 },
      { id: "extract", action: { type: "tool_exec", config: { tool: "html_extractor" } }, depends_on: ["search"], timeout_secs: 60 },
      { id: "summarize", action: { type: "llm_call", config: { model: "deepseek-chat" } }, depends_on: ["extract"], timeout_secs: 120 },
      { id: "report", action: { type: "llm_call", config: { model: "deepseek-chat", format: "markdown" } }, depends_on: ["summarize"], timeout_secs: 120 },
    ],
  },
  {
    id: "code-review",
    name: "Code Review Pipeline",
    description: "Lint → Test → Review → Report",
    steps: [
      { id: "lint", action: { type: "tool_exec", config: { tool: "linter" } }, depends_on: [], timeout_secs: 30 },
      { id: "test", action: { type: "tool_exec", config: { tool: "test_runner" } }, depends_on: ["lint"], timeout_secs: 120 },
      { id: "review", action: { type: "llm_call", config: { model: "deepseek-chat" } }, depends_on: ["test"], timeout_secs: 120 },
      { id: "report", action: { type: "notify", config: { channel: "slack" } }, depends_on: ["review"], timeout_secs: 10 },
    ],
  },
];

interface PaletteItemDef {
  type: string;
  label: string;
}

interface PaletteCategoryDef {
  category: string;
  items: PaletteItemDef[];
}

const PALETTE_CATEGORIES: PaletteCategoryDef[] = [
  { category: "📡 触发器", items: [
    { type: "timer", label: "定时器" },
    { type: "webhook", label: "Webhook" },
  ]},
  { category: "🤖 AI 处理", items: [
    { type: "llm_call", label: "LLM 调用" },
    { type: "agent_call", label: "Agent 调用" },
    { type: "analyze", label: "分析" },
  ]},
  { category: "🔍 搜索/获取", items: [
    { type: "web_search", label: "网页搜索" },
    { type: "kb_query", label: "知识库查询" },
    { type: "read_file", label: "文件读取" },
  ]},
  { category: "📤 输出", items: [
    { type: "generate_report", label: "生成报告" },
    { type: "notify", label: "推送通知" },
    { type: "write_file", label: "写文件" },
  ]},
];

function getLayoutedSteps(steps: WorkflowStep[], direction = "TB") {
  const g = new dagre.graphlib.Graph();
  g.setDefaultEdgeLabel(() => ({}));
  g.setGraph({ rankdir: direction, nodesep: 50, ranksep: 80, marginx: 40, marginy: 40 });

  const nodes: Node<WorkflowNodeData>[] = steps.map((step) => {
    g.setNode(step.id, { width: 180, height: 70 });
    return {
      id: step.id,
      type: "workflowStep",
      position: { x: 0, y: 0 },
      data: { stepName: step.id, actionType: step.action.type },
    };
  });

  const edges: Edge[] = [];
  for (const step of steps) {
    for (const dep of step.depends_on) {
      g.setEdge(dep, step.id);
      edges.push({
        id: `edge-${dep}-${step.id}`,
        source: dep,
        target: step.id,
        animated: true,
        style: { stroke: "#58a6ff", strokeWidth: 2 },
        markerEnd: { type: MarkerType.ArrowClosed, color: "#58a6ff" },
      });
    }
  }

  dagre.layout(g);

  const laidOutNodes = nodes.map((node) => {
    const dagreNode = g.node(node.id);
    return { ...node, position: { x: dagreNode.x - 90, y: dagreNode.y - 35 } };
  });

  return { nodes: laidOutNodes, edges };
}

function WorkflowStepNode({ data, selected }: NodeProps<WorkflowNodeData>) {
  const color = ACTION_COLORS[data.actionType] || ACTION_COLORS.default;
  const statusColor = data.status ? STATUS_COLORS[data.status] || STATUS_COLORS.pending : STATUS_COLORS.pending;

  return (
    <div
      style={{
        background: "#1a1d23",
        border: `2px solid ${selected ? color : "#30363d"}`,
        borderRadius: "8px",
        padding: "8px 12px",
        minWidth: "160px",
        boxShadow: selected ? `0 0 0 2px ${color}44, 0 4px 12px rgba(0,0,0,0.3)` : "0 4px 12px rgba(0,0,0,0.3)",
        position: "relative",
      }}
    >
      <Handle type="target" position={Position.Top} style={{ background: color, border: `2px solid ${color}`, width: 8, height: 8 }} />
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: 4 }}>
        <span style={{ background: color, color: "#fff", fontSize: 11, fontWeight: 600, padding: "2px 6px", borderRadius: 4 }}>
          {data.actionType}
        </span>
        {data.status && (
          <span style={{ width: 8, height: 8, borderRadius: "50%", background: statusColor, display: "inline-block" }} />
        )}
      </div>
      <div style={{ color: "#e6edf3", fontSize: 13, fontWeight: 500 }}>{data.stepName}</div>
      <Handle type="source" position={Position.Bottom} style={{ background: color, border: `2px solid ${color}`, width: 8, height: 8 }} />
    </div>
  );
}

const nodeTypes = { workflowStep: WorkflowStepNode };

function CanvasInner() {
  const reactFlowWrapper = useRef<HTMLDivElement>(null);
  const reactFlowInstance = useReactFlow();
  const typeCountRef = useRef<Record<string, number>>({});
  const [templates, setTemplates] = useState<WorkflowTemplate[]>(MOCK_TEMPLATES);
  const [selectedTemplateId, setSelectedTemplateId] = useState<string>(MOCK_TEMPLATES[0]?.id || "");
  const [templateName, setTemplateName] = useState("");
  const [templateDescription, setTemplateDescription] = useState("");
  const [nodes, setNodes, onNodesChange] = useNodesState<WorkflowNodeData>([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    api.listWorkflowTemplates?.().then((result: WorkflowTemplate[]) => {
      if (result && result.length > 0) {
        setTemplates(result);
      }
    }).catch(() => {});
  }, []);

  const loadTemplate = useCallback((template: WorkflowTemplate) => {
    setTemplateName(template.name);
    setTemplateDescription(template.description);
    const { nodes: layoutedNodes, edges: layoutedEdges } = getLayoutedSteps(template.steps);
    setNodes(layoutedNodes);
    setEdges(layoutedEdges);
  }, [setNodes, setEdges]);

  useEffect(() => {
    if (selectedTemplateId) {
      const tmpl = templates.find((t) => t.id === selectedTemplateId);
      if (tmpl) loadTemplate(tmpl);
    }
  }, [selectedTemplateId, templates, loadTemplate]);

  const onConnect = useCallback(
    (params: Connection) => {
      setEdges((eds) =>
        addEdge(
          {
            ...params,
            animated: true,
            style: { stroke: "#58a6ff", strokeWidth: 2 },
            markerEnd: { type: MarkerType.ArrowClosed, color: "#58a6ff" },
          },
          eds,
        ),
      );
    },
    [setEdges],
  );

  const onDragOver = useCallback((event: React.DragEvent) => {
    event.preventDefault();
    event.dataTransfer.dropEffect = "move";
  }, []);

  const onDrop = useCallback(
    (event: React.DragEvent) => {
      event.preventDefault();
      const type = event.dataTransfer.getData("application/reactflow");
      if (!type || !reactFlowWrapper.current) return;

      const position = reactFlowInstance.screenToFlowPosition({
        x: event.clientX,
        y: event.clientY,
      });

      typeCountRef.current[type] = (typeCountRef.current[type] || 0) + 1;
      const count = typeCountRef.current[type];

      const paletteItem = PALETTE_CATEGORIES.flatMap((c) => c.items).find((i) => i.type === type);
      const label = paletteItem?.label || type;

      const newId = `${type}-${count}`;
      const newNode: Node<WorkflowNodeData> = {
        id: newId,
        type: "workflowStep",
        position,
        data: { stepName: `${label} #${count}`, actionType: type },
      };

      setNodes((nds) => [...nds, newNode]);
    },
    [reactFlowInstance, setNodes],
  );

  const addStep = useCallback(() => {
    const newId = `step-${Date.now()}`;
    const newNode: Node<WorkflowNodeData> = {
      id: newId,
      type: "workflowStep",
      position: { x: 100, y: 80 + nodes.length * 90 },
      data: { stepName: newId, actionType: "llm_call" },
    };
    setNodes((nds) => [...nds, newNode]);
  }, [nodes.length, setNodes]);

  const deleteSelected = useCallback(() => {
    const selected = nodes.find((n) => n.selected);
    if (selected) {
      setNodes((nds) => nds.filter((n) => n.id !== selected.id));
      setEdges((eds) => eds.filter((e) => e.source !== selected.id && e.target !== selected.id));
    }
  }, [nodes, setNodes, setEdges]);

  const autoLayout = useCallback(() => {
    const steps: WorkflowStep[] = nodes.map((n) => ({
      id: n.id,
      action: { type: n.data.actionType, config: {} },
      depends_on: edges.filter((e) => e.target === n.id).map((e) => e.source),
      timeout_secs: 60,
    }));
    const { nodes: laidOut, edges: laidEdges } = getLayoutedSteps(steps);
    setNodes(laidOut);
    setEdges(laidEdges);
  }, [nodes, edges, setNodes, setEdges]);

  const saveTemplate = useCallback(async () => {
    const steps: WorkflowStep[] = nodes.map((n) => ({
      id: n.id,
      action: { type: n.data.actionType, config: {} },
      depends_on: edges.filter((e) => e.target === n.id).map((e) => e.source),
      timeout_secs: 60,
    }));
    const newTemplate: WorkflowTemplate = {
      id: `custom-${Date.now()}`,
      name: templateName || "Untitled Workflow",
      description: templateDescription,
      steps,
    };
    setLoading(true);
    try {
      if (api.saveWorkflowTemplate) {
        await api.saveWorkflowTemplate(newTemplate);
      }
      setTemplates((prev) => [...prev, newTemplate]);
      setSelectedTemplateId(newTemplate.id);
    } catch (e) {
      console.error("Save failed", e);
    } finally {
      setLoading(false);
    }
  }, [nodes, edges, templateName, templateDescription]);

  const edgesWithSelection = edges.map((e) => ({ ...e, selected: false }));

  return (
    <div>
      <div style={{ display: "flex", gap: 12, alignItems: "center", marginBottom: 12, flexWrap: "wrap" }}>
        <select
          value={selectedTemplateId}
          onChange={(e) => setSelectedTemplateId(e.target.value)}
          style={{
            background: "#0d1117", border: "1px solid #30363d", borderRadius: 4,
            padding: "6px 10px", color: "#e6edf3", fontSize: 13, minWidth: 200,
          }}
        >
          {templates.map((tmpl) => (
            <option key={tmpl.id} value={tmpl.id}>{tmpl.name}</option>
          ))}
        </select>

        <input
          value={templateName}
          onChange={(e) => setTemplateName(e.target.value)}
          placeholder="Workflow name"
          style={{
            background: "#0d1117", border: "1px solid #30363d", borderRadius: 4,
            padding: "6px 10px", color: "#e6edf3", fontSize: 13, width: 200,
          }}
        />
        <input
          value={templateDescription}
          onChange={(e) => setTemplateDescription(e.target.value)}
          placeholder="Description"
          style={{
            background: "#0d1117", border: "1px solid #30363d", borderRadius: 4,
            padding: "6px 10px", color: "#e6edf3", fontSize: 13, width: 240,
          }}
        />

        <button onClick={addStep} style={{ padding: "6px 12px", fontSize: 12, background: "#1f6feb", color: "#fff", border: "none", borderRadius: 4, cursor: "pointer" }}>
          + Add Step
        </button>
        <button onClick={deleteSelected} style={{ padding: "6px 12px", fontSize: 12, background: "#da3633", color: "#fff", border: "none", borderRadius: 4, cursor: "pointer" }}>
          Delete
        </button>
        <button onClick={autoLayout} style={{ padding: "6px 12px", fontSize: 12, background: "#21262d", color: "#e6edf3", border: "1px solid #30363d", borderRadius: 4, cursor: "pointer" }}>
          Auto Layout
        </button>
        <button
          onClick={saveTemplate}
          disabled={loading}
          style={{
            padding: "6px 16px", fontSize: 12, background: loading ? "#21262d" : "#238636",
            color: "#fff", border: "none", borderRadius: 4, cursor: loading ? "default" : "pointer",
          }}
        >
          {loading ? "Saving..." : "Save Workflow"}
        </button>
      </div>

      <div style={{ display: "flex", height: 500, borderRadius: 8, overflow: "hidden", border: "1px solid #30363d" }}>
        <aside
          style={{
            width: 200,
            background: "#161b22",
            borderRight: "1px solid #30363d",
            padding: "12px",
            overflowY: "auto",
            flexShrink: 0,
          }}
          onDragStart={(event) => {
            const type = (event.currentTarget as HTMLElement).dataset?.type;
            if (type) event.dataTransfer.setData("application/reactflow", type);
          }}
        >
          {PALETTE_CATEGORIES.map((cat) => (
            <div key={cat.category}>
              <div style={{ fontSize: 11, fontWeight: 600, color: "#8b949e", textTransform: "uppercase", margin: "12px 0 4px 0" }}>
                {cat.category}
              </div>
              {cat.items.map((item) => (
                <div
                  key={item.type}
                  data-type={item.type}
                  draggable
                  style={{
                    padding: "6px 8px",
                    margin: "2px 0",
                    fontSize: 13,
                    color: "#e6edf3",
                    background: "rgba(255,255,255,0.04)",
                    border: "1px solid #30363d",
                    borderRadius: 4,
                    cursor: "grab",
                    userSelect: "none",
                    transition: "background 0.15s",
                    borderLeft: `3px solid ${ACTION_COLORS[item.type] || ACTION_COLORS.default}`,
                  }}
                  onDragStart={(e) => {
                    e.dataTransfer.setData("application/reactflow", item.type);
                    e.dataTransfer.effectAllowed = "move";
                  }}
                >
                  {item.label}
                </div>
              ))}
            </div>
          ))}
        </aside>

        <div ref={reactFlowWrapper} style={{ flex: 1, position: "relative" }}>
          <ReactFlow
            nodes={nodes}
            edges={edgesWithSelection}
            onNodesChange={onNodesChange as (changes: NodeChange[]) => void}
            onEdgesChange={onEdgesChange as (changes: EdgeChange[]) => void}
            onConnect={onConnect}
            onDrop={onDrop}
            onDragOver={onDragOver}
            nodeTypes={nodeTypes}
            fitView
            minZoom={0.2}
            maxZoom={3}
            style={{ background: "#0d1117" }}
            deleteKeyCode={["Backspace", "Delete"]}
          >
            <Controls style={{ background: "#161b22", border: "1px solid #30363d", borderRadius: 6 }} />
            <Background color="#30363d" gap={20} />
            <MiniMap
              style={{ background: "#161b22", border: "1px solid #30363d", borderRadius: 6 }}
              nodeColor={() => "#58a6ff"}
            />
          </ReactFlow>
        </div>
      </div>

      {nodes.length === 0 && (
        <div style={{ textAlign: "center", padding: 24, color: "#8b949e", fontSize: 13 }}>
          Select a template above or add steps manually.
        </div>
      )}
    </div>
  );
}

export default function WorkflowBuilder() {
  return (
    <ReactFlowProvider>
      <CanvasInner />
    </ReactFlowProvider>
  );
}