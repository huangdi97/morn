import { useState } from "react";
import { api } from "../api";

const ITEM_TYPES = ["agent", "tool", "knowledge", "skill", "persona", "workflow", "team_template"];

export default function StudioPublisher() {
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [itemType, setItemType] = useState("agent");
  const [price, setPrice] = useState(0);
  const [author, setAuthor] = useState("");
  const [isFree, setIsFree] = useState(true);
  const [publishing, setPublishing] = useState(false);
  const [result, setResult] = useState<{ ok: boolean; msg: string } | null>(null);

  const handlePublish = async () => {
    if (!name.trim() || !description.trim()) {
      setResult({ ok: false, msg: "Name and description required" });
      return;
    }
    setPublishing(true);
    setResult(null);
    try {
      const id = await api.hubPublish({
        name: name.trim(),
        description: description.trim(),
        itemType,
        price: isFree ? 0 : price,
        author: author.trim() || "Anonymous",
      });
      setResult({ ok: true, msg: `Published successfully! ID: ${id}` });
      setName("");
      setDescription("");
      setPrice(0);
    } catch (e: any) {
      setResult({ ok: false, msg: `Publish failed: ${e}` });
    } finally {
      setPublishing(false);
    }
  };

  return (
    <div>
      <h2 style={{ color: "#e6edf3", marginBottom: "16px" }}>Publish to Hub</h2>

      <div style={{ display: "flex", flexDirection: "column", gap: "12px", maxWidth: "480px" }}>
        <input type="text" placeholder="Component name" value={name}
          onChange={e => setName(e.target.value)}
          style={inputStyle} />

        <textarea placeholder="Description" value={description}
          onChange={e => setDescription(e.target.value)} rows={3}
          style={{ ...inputStyle, resize: "vertical" }} />

        <select value={itemType} onChange={e => setItemType(e.target.value)}
          style={inputStyle}>
          {ITEM_TYPES.map(t => <option key={t} value={t}>{t}</option>)}
        </select>

        <input type="text" placeholder="Author name" value={author}
          onChange={e => setAuthor(e.target.value)}
          style={inputStyle} />

        <div style={{ display: "flex", alignItems: "center", gap: "12px" }}>
          <label style={{ color: "#e6edf3", fontSize: "13px", display: "flex", alignItems: "center", gap: "4px" }}>
            <input type="checkbox" checked={isFree} onChange={e => setIsFree(e.target.checked)} />
            Publish as Free
          </label>
          {!isFree && (
            <input type="number" min={0} step={0.001} value={price}
              onChange={e => setPrice(parseFloat(e.target.value) || 0)}
              placeholder="Price"
              style={{ ...inputStyle, width: "120px" }} />
          )}
        </div>

        <button onClick={handlePublish} disabled={publishing}
          style={{
            padding: "8px 16px", background: publishing ? "#21262d" : "#1f6feb",
            color: "#fff", border: "none", borderRadius: "4px",
            cursor: publishing ? "default" : "pointer", fontSize: "13px",
          }}>
          {publishing ? "Publishing..." : "Publish to Hub"}
        </button>

        {result && (
          <div style={{
            padding: "8px 12px", borderRadius: "4px", fontSize: "13px",
            background: result.ok ? "#0d2818" : "#3d1111",
            color: result.ok ? "#3fb950" : "#f85149",
          }}>
            {result.msg}
          </div>
        )}
      </div>
    </div>
  );
}

const inputStyle: React.CSSProperties = {
  background: "#0d1117", border: "1px solid #30363d", borderRadius: "4px",
  padding: "6px 12px", color: "#e6edf3", fontSize: "13px", width: "100%",
  boxSizing: "border-box",
};