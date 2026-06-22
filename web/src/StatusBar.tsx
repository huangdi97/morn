import { useState, useEffect } from "react";
import { api } from "./api";
import { RenderSlot } from "./slots/RenderSlot";

export default function StatusBar() {
  const [status, setStatus] = useState("");
  const [cost, setCost] = useState("¥0");

  useEffect(() => {
    api.getStatus().then((s: any) => {
      setStatus(`v${s.version} | ${s.turn_count} turns`);
    }).catch(() => {});
    api.getCostSummary().then((res: any) => {
      let total = 0;
      if (typeof res === "string") {
        try { total = JSON.parse(res).total ?? 0; } catch { total = 0; }
      } else {
        total = res?.total ?? 0;
      }
      setCost(`¥${total}`);
    }).catch(() => {});
  }, []);

  return (
    <div style={{
      display: "flex",
      alignItems: "center",
      justifyContent: "center",
      gap: "16px",
      padding: "4px 16px",
      background: "var(--bg-secondary)",
      borderTop: "1px solid var(--border)",
      fontSize: "11px",
      color: "var(--text-tertiary)",
      fontFamily: "var(--font-mono)",
      flexShrink: 0,
    }}>
      <span>🟢 {status || "Loading..."}</span>
      <span>💰 {cost}</span>
      <span>⚡ Provider</span>
      <RenderSlot name="status-bar" />
    </div>
  );
}