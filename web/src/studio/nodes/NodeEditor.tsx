import { useState, useCallback } from "react";
import {
  NodeDefinition,
  NodeEdge,
  NodeGraph,
  NodeTemplate,
  NODE_COLORS,
  NODE_CATEGORIES,
} from "./NodeType";
import { NODE_TEMPLATES } from "./NodeRegistry";

interface NodeEditorProps {
  initialGraph?: NodeGraph;
  onGraphChange?: (graph: NodeGraph) => void;
}

let nodeIdCounter = 0;
function generateId(): string {
  nodeIdCounter += 1;
  return `node_${nodeIdCounter}_${Date.now()}`;
}

function generateEdgeId(): string {
  return `edge_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`;
}

export function NodeEditor({ initialGraph, onGraphChange }: NodeEditorProps) {
  const [nodes, setNodes] = useState<NodeDefinition[]>(initialGraph?.nodes ?? []);
  const [edges, setEdges] = useState<NodeEdge[]>(initialGraph?.edges ?? []);
  const [selectedNode, setSelectedNode] = useState<string | null>(null);
  const [selectedTemplate, setSelectedTemplate] = useState<NodeTemplate | null>(null);
  const [connectingFrom, setConnectingFrom] = useState<{ nodeId: string; output: string } | null>(null);

  const addNode = useCallback(
    (template: NodeTemplate) => {
      const newNode: NodeDefinition = {
        id: generateId(),
        nodeType: template.nodeType,
        label: template.label,
        config: { ...template.defaultConfig },
        inputs: [...template.inputs],
        outputs: [...template.outputs],
      };
      setNodes((prev) => [...prev, newNode]);
      onGraphChange?.({ nodes: [...nodes, newNode], edges });
    },
    [nodes, edges, onGraphChange],
  );

  const removeNode = useCallback(
    (nodeId: string) => {
      setNodes((prev) => prev.filter((n) => n.id !== nodeId));
      setEdges((prev) =>
        prev.filter((e) => e.source !== nodeId && e.target !== nodeId),
      );
      setSelectedNode((prev) => (prev === nodeId ? null : prev));
    },
    [],
  );

  const updateNodeConfig = useCallback(
    (nodeId: string, key: string, value: unknown) => {
      setNodes((prev) =>
        prev.map((n) =>
          n.id === nodeId
            ? { ...n, config: { ...n.config, [key]: value } }
            : n,
        ),
      );
    },
    [],
  );

  const addEdge = useCallback(
    (source: string, sourceOutput: string, target: string, targetInput: string) => {
      if (source === target) return;
      const exists = edges.some(
        (e) => e.source === source && e.target === target,
      );
      if (exists) return;
      const newEdge: NodeEdge = {
        id: generateEdgeId(),
        source,
        sourceOutput,
        target,
        targetInput,
      };
      setEdges((prev) => [...prev, newEdge]);
    },
    [edges],
  );

  const removeEdge = useCallback((edgeId: string) => {
    setEdges((prev) => prev.filter((e) => e.id !== edgeId));
  }, []);

  const startConnection = useCallback((nodeId: string, output: string) => {
    setConnectingFrom({ nodeId, output });
  }, []);

  const completeConnection = useCallback(
    (targetNodeId: string, targetInput: string) => {
      if (connectingFrom) {
        addEdge(connectingFrom.nodeId, connectingFrom.output, targetNodeId, targetInput);
        setConnectingFrom(null);
      }
    },
    [connectingFrom, addEdge],
  );

  const canvasNode = useCallback(
    (node: NodeDefinition) => {
      const color = NODE_COLORS[node.nodeType] || "#666";
      const isSelected = selectedNode === node.id;
      return (
        <div
          key={node.id}
          onClick={() => setSelectedNode(node.id)}
          style={{
            padding: "10px 14px",
            margin: "6px 0",
            borderRadius: "8px",
            border: `2px solid ${isSelected ? color : "#30363d"}`,
            background: "#1a1d23",
            cursor: "pointer",
            position: "relative",
          }}
        >
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: "8px",
              marginBottom: "6px",
            }}
          >
            <span
              style={{
                width: "10px",
                height: "10px",
                borderRadius: "50%",
                background: color,
                display: "inline-block",
              }}
            />
            <span style={{ color: "#e6edf3", fontSize: "13px", fontWeight: 600 }}>
              {node.label}
            </span>
            <span style={{ color: "#8b949e", fontSize: "11px", marginLeft: "auto" }}>
              {node.nodeType}
            </span>
          </div>

          <div style={{ display: "flex", gap: "4px", flexWrap: "wrap" }}>
            {node.inputs.map((input) => (
              <span
                key={`in-${input}`}
                onClick={(e) => {
                  e.stopPropagation();
                  completeConnection(node.id, input);
                }}
                style={{
                  fontSize: "10px",
                  padding: "2px 6px",
                  borderRadius: "4px",
                  background: `${color}33`,
                  color,
                  cursor: "pointer",
                  border: connectingFrom ? `1px dashed ${color}` : "none",
                }}
                title={`Input: ${input}`}
              >
                {connectingFrom ? `→ ${input}` : `in:${input}`}
              </span>
            ))}
            {node.outputs.map((output) => (
              <span
                key={`out-${output}`}
                onClick={(e) => {
                  e.stopPropagation();
                  startConnection(node.id, output);
                }}
                style={{
                  fontSize: "10px",
                  padding: "2px 6px",
                  borderRadius: "4px",
                  background: `${color}44`,
                  color,
                  cursor: "pointer",
                }}
                title={`Output: ${output}`}
              >
                {`out:${output}`}
              </span>
            ))}
          </div>

          {isSelected && (
            <div style={{ marginTop: "8px", borderTop: "1px solid #30363d", paddingTop: "8px" }}>
              {Object.entries(node.config).map(([key, value]) => (
                <div
                  key={key}
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: "6px",
                    marginBottom: "4px",
                  }}
                >
                  <label
                    style={{ color: "#8b949e", fontSize: "11px", minWidth: "80px" }}
                  >
                    {key}
                  </label>
                  <input
                    value={String(value)}
                    onChange={(e) => updateNodeConfig(node.id, key, e.target.value)}
                    style={{
                      flex: 1,
                      padding: "3px 6px",
                      borderRadius: "4px",
                      border: "1px solid #30363d",
                      background: "#0d1117",
                      color: "#e6edf3",
                      fontSize: "11px",
                    }}
                  />
                </div>
              ))}
              <button
                onClick={() => removeNode(node.id)}
                style={{
                  marginTop: "4px",
                  fontSize: "11px",
                  padding: "3px 8px",
                  background: "#ef4444",
                  color: "#fff",
                  border: "none",
                  borderRadius: "4px",
                  cursor: "pointer",
                }}
              >
                Delete Node
              </button>
            </div>
          )}
        </div>
      );
    },
    [selectedNode, connectingFrom, updateNodeConfig, removeNode, startConnection, completeConnection],
  );

  const canvasEdges = edges.map((edge) => (
    <div
      key={edge.id}
      onClick={() => removeEdge(edge.id)}
      style={{
        padding: "4px 8px",
        margin: "2px 0",
        borderRadius: "4px",
        background: "#161b22",
        fontSize: "11px",
        color: "#8b949e",
        cursor: "pointer",
        display: "flex",
        alignItems: "center",
        gap: "6px",
      }}
      title="Click to remove"
    >
      <span style={{ color: "#58a6ff" }}>{edge.source}</span>
      <span style={{ color: "#8b949e" }}>→</span>
      <span style={{ color: "#22c55e" }}>{edge.target}</span>
    </div>
  ));

  return (
    <div
      style={{
        display: "flex",
        gap: "12px",
        height: "100%",
        minHeight: "500px",
      }}
    >
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
        <h4 style={{ color: "#e6edf3", margin: "0 0 12px 0", fontSize: "14px" }}>
          Node Templates
        </h4>
        {NODE_CATEGORIES.map((cat) => (
          <div key={cat.name} style={{ marginBottom: "12px" }}>
            <div
              style={{
                color: cat.color,
                fontSize: "11px",
                marginBottom: "4px",
                textTransform: "uppercase",
                letterSpacing: "0.5px",
                fontWeight: 600,
              }}
            >
              {cat.name}
            </div>
            {NODE_TEMPLATES.filter((t) => t.category === cat.name).map((template) => (
              <div
                key={template.label}
                onClick={() => {
                  setSelectedTemplate(template);
                  addNode(template);
                }}
                style={{
                  padding: "6px 8px",
                  marginBottom: "3px",
                  borderRadius: "5px",
                  background:
                    selectedTemplate?.label === template.label
                      ? `${cat.color}22`
                      : "#1a1d23",
                  border: `1px solid ${
                    selectedTemplate?.label === template.label
                      ? cat.color
                      : "#30363d"
                  }`,
                  cursor: "pointer",
                  fontSize: "12px",
                  color: "#e6edf3",
                  transition: "all 0.15s",
                }}
              >
                <div style={{ fontWeight: 500 }}>{template.label}</div>
                <div
                  style={{ color: "#8b949e", fontSize: "10px", marginTop: "2px" }}
                >
                  {template.description}
                </div>
              </div>
            ))}
          </div>
        ))}
      </div>

      <div
        style={{
          flex: 1,
          background: "#0d1117",
          borderRadius: "8px",
          border: "1px solid #30363d",
          padding: "12px",
          overflowY: "auto",
        }}
      >
        <div
          style={{
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
            marginBottom: "12px",
          }}
        >
          <h4 style={{ color: "#e6edf3", margin: 0, fontSize: "14px" }}>
            Canvas ({nodes.length} nodes, {edges.length} edges)
          </h4>
          <div style={{ display: "flex", gap: "6px" }}>
            {connectingFrom && (
              <button
                onClick={() => setConnectingFrom(null)}
                style={{
                  fontSize: "11px",
                  padding: "3px 8px",
                  background: "#f59e0b",
                  color: "#000",
                  border: "none",
                  borderRadius: "4px",
                  cursor: "pointer",
                }}
              >
                Cancel Connection
              </button>
            )}
            <button
              onClick={() => {
                setNodes([]);
                setEdges([]);
                setSelectedNode(null);
                onGraphChange?.({ nodes: [], edges: [] });
              }}
              style={{
                fontSize: "11px",
                padding: "3px 8px",
                background: "#ef4444",
                color: "#fff",
                border: "none",
                borderRadius: "4px",
                cursor: "pointer",
              }}
            >
              Clear
            </button>
          </div>
        </div>

        {connectingFrom && (
          <div
            style={{
              padding: "6px 10px",
              marginBottom: "8px",
              borderRadius: "4px",
              background: "#f59e0b22",
              border: "1px dashed #f59e0b",
              color: "#f59e0b",
              fontSize: "12px",
            }}
          >
            Connecting from {connectingFrom.nodeId}:{connectingFrom.output} —
            click an input handle to complete
          </div>
        )}

        <div style={{ display: "flex", gap: "16px" }}>
          <div style={{ flex: 1 }}>
            {nodes.length === 0 ? (
              <div
                style={{
                  color: "#8b949e",
                  fontSize: "13px",
                  textAlign: "center",
                  padding: "40px",
                }}
              >
                Select a template from the left panel to add nodes to the canvas
              </div>
            ) : (
              nodes.map(canvasNode)
            )}
          </div>
          {edges.length > 0 && (
            <div
              style={{
                width: "200px",
                flexShrink: 0,
              }}
            >
              <div
                style={{
                  color: "#8b949e",
                  fontSize: "11px",
                  marginBottom: "6px",
                  textTransform: "uppercase",
                }}
              >
                Connections
              </div>
              {canvasEdges}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
