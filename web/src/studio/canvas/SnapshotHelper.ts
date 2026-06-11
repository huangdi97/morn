export type CanvasSnapshot = {
  nodes: Node<NodeData>[];
  edges: Edge[];
};

import type { Edge, Node } from "reactflow";

interface NodeData {
  nodeType: string;
  label: string;
  name: string;
  detail: string;
  snapshot?: Record<string, unknown>;
}

export function cloneSnapshot(nodes: Node<NodeData>[], edges: Edge[]): CanvasSnapshot {
  return {
    nodes: JSON.parse(JSON.stringify(nodes)) as Node<NodeData>[],
    edges: JSON.parse(JSON.stringify(edges)) as Edge[],
  };
}

function escapeXml(value: string) {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}

export function downloadText(filename: string, content: string, mimeType: string) {
  const blob = new Blob([content], { type: mimeType });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = filename;
  link.click();
  URL.revokeObjectURL(url);
}

const NODE_COLORS: Record<string, string> = {
  persona: "#7c3aed",
  tool: "#2563eb",
  knowledge: "#16a34a",
  skill: "#d97706",
  model: "#dc2626",
  agent: "#0891b2",
};

export function buildSnapshotSvg(nodes: Node<NodeData>[], edges: Edge[]) {
  const minX = Math.min(...nodes.map((node) => node.position.x), 0) - 80;
  const minY = Math.min(...nodes.map((node) => node.position.y), 0) - 80;
  const maxX = Math.max(...nodes.map((node) => node.position.x + 180), 760) + 80;
  const maxY = Math.max(...nodes.map((node) => node.position.y + 90), 420) + 80;
  const width = maxX - minX;
  const height = maxY - minY;
  const nodeById = new Map(nodes.map((node) => [node.id, node]));
  const edgeLines = edges
    .map((edge) => {
      const source = nodeById.get(edge.source);
      const target = nodeById.get(edge.target);
      if (!source || !target) return "";
      const x1 = source.position.x - minX + 180;
      const y1 = source.position.y - minY + 40;
      const x2 = target.position.x - minX;
      const y2 = target.position.y - minY + 40;
      return `<line x1="${x1}" y1="${y1}" x2="${x2}" y2="${y2}" stroke="#58a6ff" stroke-width="2" marker-end="url(#arrow)" />`;
    })
    .join("");
  const nodeBoxes = nodes
    .map((node) => {
      const x = node.position.x - minX;
      const y = node.position.y - minY;
      const color = NODE_COLORS[node.data.nodeType] || "#666";
      const title = escapeXml(node.data.name || node.data.label || node.id);
      const detail = escapeXml(node.data.detail || "");
      return `<g><rect x="${x}" y="${y}" width="180" height="80" rx="8" fill="#1a1d23" stroke="${color}" stroke-width="2"/><rect x="${x + 12}" y="${y + 10}" width="76" height="20" rx="4" fill="${color}"/><text x="${x + 20}" y="${y + 25}" fill="#fff" font-size="12" font-family="Arial">${escapeXml(node.data.label)}</text><text x="${x + 12}" y="${y + 52}" fill="#e6edf3" font-size="13" font-family="Arial">${title}</text><text x="${x + 12}" y="${y + 69}" fill="#8b949e" font-size="11" font-family="Arial">${detail.slice(0, 28)}</text></g>`;
    })
    .join("");

  return `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}" viewBox="0 0 ${width} ${height}"><defs><marker id="arrow" markerWidth="10" markerHeight="10" refX="8" refY="3" orient="auto"><path d="M0,0 L0,6 L9,3 z" fill="#58a6ff"/></marker></defs><rect width="100%" height="100%" fill="#0d1117"/>${edgeLines}${nodeBoxes}</svg>`;
}