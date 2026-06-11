interface EditorPanelProps {
  selectedNode: { id: string; data: { name: string; label: string; detail: string } } | null;
  onUpdate: (patch: { name?: string; detail?: string }) => void;
  onDelete: () => void;
}

export function EditorPanel({ selectedNode, onUpdate, onDelete }: EditorPanelProps) {
  if (!selectedNode) return null;

  return (
    <div
      style={{
        marginTop: "12px",
        paddingTop: "12px",
        borderTop: "1px solid #30363d",
        display: "flex",
        flexDirection: "column",
        gap: "8px",
      }}
    >
      <div style={{ color: "#e6edf3", fontSize: "13px", fontWeight: 600 }}>{selectedNode.data.label} 设置</div>
      <input
        value={selectedNode.data.name}
        onChange={(event) => onUpdate({ name: event.target.value })}
        style={{ background: "#0d1117", border: "1px solid #30363d", color: "#e6edf3", borderRadius: "6px", padding: "7px 8px" }}
      />
      <textarea
        value={selectedNode.data.detail}
        onChange={(event) => onUpdate({ detail: event.target.value })}
        rows={3}
        style={{ background: "#0d1117", border: "1px solid #30363d", color: "#e6edf3", borderRadius: "6px", padding: "7px 8px", resize: "vertical" }}
      />
      <button onClick={onDelete} style={{ fontSize: "12px", padding: "6px", background: "#dc2626" }}>
        删除节点
      </button>
    </div>
  );
}