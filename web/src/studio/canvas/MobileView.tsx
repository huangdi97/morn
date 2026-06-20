import { useState, useEffect, type ReactNode } from "react";
import type { Node } from "reactflow";
import EditorPanel from "./EditorPanel";
import { NodeData } from "../types";
import { useTranslation } from '../../i18n';

export function useSmallScreen() {
  const [isSmallScreen, setIsSmallScreen] = useState(() => window.innerWidth < 768);

  useEffect(() => {
    const media = window.matchMedia("(max-width: 767px)");
    const update = () => setIsSmallScreen(media.matches);
    update();
    media.addEventListener("change", update);
    return () => media.removeEventListener("change", update);
  }, []);

  return isSmallScreen;
}

const NODE_COLORS: Record<string, string> = {
  persona: "#7c3aed",
  tool: "#2563eb",
  knowledge: "#16a34a",
  skill: "#d97706",
  model: "#dc2626",
  agent: "#0891b2",
};

interface PaletteItem {
  type: string;
  label: string;
  color: string;
}

interface MobileViewProps {
  nodes: Node<NodeData>[];
  selectedNodeId: string | null;
  historyCounts: { undo: number; redo: number };
  paletteItems: PaletteItem[];
  onAddNode: (type: string) => void;
  onUndo: () => void;
  onRedo: () => void;
  onSnapshot: () => void;
  onExport: () => void;
  onSelectNode: (id: string | null) => void;
  onUpdateNode: (patch: Partial<NodeData>) => void;
  onDeleteNode: () => void;
}

export function MobileView({
  nodes,
  selectedNodeId,
  historyCounts,
  paletteItems,
  onAddNode,
  onUndo,
  onRedo,
  onSnapshot,
  onExport,
  onSelectNode,
  onUpdateNode,
  onDeleteNode,
  children,
}: MobileViewProps & { children: ReactNode }) {
  const isSmallScreen = useSmallScreen();
  const { t } = useTranslation();
  if (!isSmallScreen) return <>{children}</>;

  const selectedNode = nodes.find((n) => n.id === selectedNodeId) ?? null;

  return (
    <div style={{ display: "flex", flexDirection: "column", minHeight: "600px", gap: "12px" }}>
      <div style={{ display: "flex", gap: "6px", overflowX: "auto", paddingBottom: "4px" }}>
        {paletteItems.map((item) => (
          <button
            key={item.type}
            onClick={() => onAddNode(item.type)}
            style={{ flex: "0 0 auto", border: `1px solid ${item.color}`, background: "#161b22", color: "#e6edf3", padding: "8px 10px", borderRadius: "6px" }}
          >
            {item.label}
          </button>
        ))}
      </div>
      <div style={{ display: "flex", gap: "6px" }}>
        <button onClick={onUndo} disabled={historyCounts.undo === 0} style={{ flex: 1, opacity: historyCounts.undo === 0 ? 0.5 : 1 }}>
          {t('node_canvas.undo')}
        </button>
        <button onClick={onRedo} disabled={historyCounts.redo === 0} style={{ flex: 1, opacity: historyCounts.redo === 0 ? 0.5 : 1 }}>
          {t('node_canvas.redo')}
        </button>
        <button onClick={onSnapshot} style={{ flex: 1 }}>
          {t('node_canvas.snapshot')}
        </button>
      </div>
      <div style={{ background: "#161b22", border: "1px solid #30363d", borderRadius: "8px", overflow: "hidden" }}>
        {nodes.map((node) => (
          <button
            key={node.id}
            onClick={() => onSelectNode(node.id)}
            style={{
              width: "100%",
              display: "flex",
              alignItems: "center",
              gap: "10px",
              padding: "12px",
              border: 0,
              borderBottom: "1px solid #30363d",
              background: node.id === selectedNodeId ? "#1f2937" : "#161b22",
              color: "#e6edf3",
              textAlign: "left",
            }}
          >
            <span style={{ width: "10px", height: "10px", borderRadius: "50%", background: NODE_COLORS[node.data.nodeType] || "#666" }} />
            <span style={{ fontSize: "13px", fontWeight: 600 }}>{node.data.name}</span>
            <span style={{ color: "#8b949e", fontSize: "12px" }}>{node.data.label}</span>
          </button>
        ))}
      </div>
      <div style={{ background: "#161b22", border: "1px solid #30363d", borderRadius: "8px", padding: "12px" }}>
        <EditorPanel
          selectedNode={selectedNode}
          selectedEdgeId={null}
          onUpdate={onUpdateNode}
          onDelete={onDeleteNode}
        />
        <button onClick={onExport} style={{ width: "100%", marginTop: "12px", padding: "8px" }}>
          {t('node_canvas.export_def')}
        </button>
      </div>
    </div>
  );
}