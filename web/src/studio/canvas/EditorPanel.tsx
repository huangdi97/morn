import { useTranslation } from '../../i18n';

interface EditorPanelProps {
  selectedNode: { id: string; data: { name: string; label: string; detail: string; nodeType: string; expanded?: boolean } } | null;
  selectedEdgeId: string | null;
  onUpdate: (patch: { name?: string; detail?: string; expanded?: boolean }) => void;
  onDelete: () => void;
}

export default function EditorPanel({ selectedNode, selectedEdgeId, onUpdate, onDelete }: EditorPanelProps) {
  const { t } = useTranslation();
  if (!selectedNode) return null;

  if (selectedEdgeId) {
    return (
      <div style={{ marginTop: "12px", paddingTop: "12px", borderTop: "1px solid #30363d", display: "flex", flexDirection: "column", gap: "8px" }}>
        <div style={{ color: "#e6edf3", fontSize: "13px", fontWeight: 600 }}>{t('node_canvas.edge_settings')}</div>
        <div style={{ color: "#8b949e", fontSize: "12px" }}>ID: {selectedEdgeId}</div>
        <button onClick={onDelete} style={{ fontSize: "12px", padding: "6px", background: "#dc2626" }}>{t('node_canvas.delete_edge')}</button>
      </div>
    );
  }

  return (
    <div style={{ marginTop: "12px", paddingTop: "12px", borderTop: "1px solid #30363d", display: "flex", flexDirection: "column", gap: "8px" }}>
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
        <div style={{ color: "#e6edf3", fontSize: "13px", fontWeight: 600 }}>{t('node_canvas.node_settings', { label: selectedNode.data.label })}</div>
        <button onClick={() => onUpdate({ expanded: !selectedNode.data.expanded })}
          style={{ fontSize: "11px", padding: "2px 6px", background: "transparent", border: "1px solid #30363d", color: "#e6edf3", borderRadius: "4px", cursor: "pointer" }}>
          {selectedNode.data.expanded ? t('node_canvas.collapse') : t('node_canvas.expand')}
        </button>
      </div>
      <label style={{ color: "#8b949e", fontSize: "11px" }}>{t('node_canvas.name_label')}</label>
      <input value={selectedNode.data.name} onChange={(event) => onUpdate({ name: event.target.value })}
        style={{ background: "#0d1117", border: "1px solid #30363d", color: "#e6edf3", borderRadius: "6px", padding: "7px 8px" }} />
      {selectedNode.data.expanded && (
        <>
          <label style={{ color: "#8b949e", fontSize: "11px" }}>{t('node_canvas.detail_config')}</label>
          <textarea value={selectedNode.data.detail} onChange={(event) => onUpdate({ detail: event.target.value })}
            rows={4} style={{ background: "#0d1117", border: "1px solid #30363d", color: "#e6edf3", borderRadius: "6px", padding: "7px 8px", resize: "vertical" }} />
          <div style={{ color: "#6b7280", fontSize: "10px" }}>{t('node_canvas.type_value', { type: selectedNode.data.nodeType })}</div>
        </>
      )}
      <button onClick={onDelete} style={{ fontSize: "12px", padding: "6px", background: "#dc2626" }}>{t('node_canvas.delete_node')}</button>
    </div>
  );
}