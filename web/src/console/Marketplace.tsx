import { useState, useEffect } from "react";
import { api } from "../api";

interface Listing {
  id: string;
  name: string;
  item_type: string;
  description: string;
  price: number;
  author: string;
  rating: number;
  reviewCount: number;
  downloads: number;
}

const hardcodedListings: Listing[] = [
  { id: "l1", name: "Web Search Pro", item_type: "tool", description: "Advanced web search", price: 0.001, author: "Morn Labs", rating: 4.5, reviewCount: 42, downloads: 1230 },
  { id: "l2", name: "Stock Market Data", item_type: "knowledge", description: "Real-time stock quotes", price: 0.01, author: "Morn Labs", rating: 4.2, reviewCount: 28, downloads: 890 },
  { id: "l3", name: "Deep Research Skill", item_type: "skill", description: "Multi-step research", price: 0.01, author: "Morn Labs", rating: 4.8, reviewCount: 15, downloads: 560 },
  { id: "l4", name: "Research Agent", item_type: "agent", description: "Full-featured research agent", price: 0.05, author: "Morn Labs", rating: 4.6, reviewCount: 33, downloads: 340 },
  { id: "l5", name: "Weekly Report", item_type: "workflow", description: "Auto report generation", price: 0.03, author: "Morn Labs", rating: 4.1, reviewCount: 8, downloads: 120 },
];

export default function Marketplace() {
  const [listings, setListings] = useState<Listing[]>(hardcodedListings);
  const [tab, setTab] = useState("all");

  useEffect(() => {
    api.getMarketListings(null).then(setListings).catch(() => {
      setListings(hardcodedListings);
    });
  }, []);

  const filtered = tab === "all" ? listings : listings.filter(l => l.item_type === tab);

  const getTypeColor = (type: string) => {
    switch (type) {
      case "tool": return "#3fb950";
      case "knowledge": return "#bc8cff";
      case "skill": return "#d29922";
      case "agent": return "#58a6ff";
      case "workflow": return "#f0883e";
      default: return "#8b949e";
    }
  };

  const tabs = ["all", "tool", "knowledge", "skill", "agent", "workflow"];

  return (
    <div>
      <h2 style={{ color: "#e6edf3", marginBottom: "16px" }}>Marketplace</h2>

      <div style={{ display: "flex", gap: "8px", marginBottom: "16px", flexWrap: "wrap" }}>
        {tabs.map(t => (
          <button key={t} onClick={() => setTab(t)}
            style={{
              background: tab === t ? "#1f6feb" : "transparent",
              color: tab === t ? "#fff" : "#8b949e",
              border: "1px solid #30363d",
              padding: "4px 12px",
              borderRadius: "4px",
              cursor: "pointer",
              textTransform: "capitalize",
              fontSize: "13px",
            }}>
            {t}
          </button>
        ))}
      </div>

      <div style={{ display: "grid", gridTemplateColumns: "repeat(2, 1fr)", gap: "12px" }}>
        {filtered.map(listing => (
          <div key={listing.id} className="market-card">
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
              <span style={{ color: getTypeColor(listing.item_type), fontSize: "11px", textTransform: "uppercase" }}>
                {listing.item_type}
              </span>
              <span style={{ display: "flex", alignItems: "center", gap: "4px" }}>
                <span style={{ color: "#d29922", fontSize: "13px" }}>{"★".repeat(Math.round(listing.rating))}{"☆".repeat(5 - Math.round(listing.rating))}</span>
                <span style={{ color: "#8b949e", fontSize: "11px" }}>({listing.reviewCount})</span>
              </span>
            </div>
            <div style={{ color: "#e6edf3", fontWeight: "bold", marginTop: "8px" }}>{listing.name}</div>
            <div style={{ color: "#8b949e", fontSize: "13px", marginTop: "4px" }}>{listing.description}</div>
            <div style={{ display: "flex", justifyContent: "space-between", marginTop: "12px" }}>
              <span style={{ color: "#8b949e", fontSize: "12px" }}>By {listing.author} · {listing.downloads} downloads</span>
              {listing.price === 0 ? (
                <span style={{ color: "#3fb950", fontSize: "11px", fontWeight: "bold", background: "rgba(63,185,80,0.15)", padding: "2px 8px", borderRadius: "4px" }}>免费</span>
              ) : (
                <span style={{ color: "#f85149", fontWeight: "bold", fontSize: "13px" }}>¥{listing.price.toFixed(3)}</span>
              )}
            </div>
            <button style={{
              width: "100%", marginTop: "12px", padding: "6px",
              background: "#1f6feb", color: "#fff", border: "none",
              borderRadius: "4px", cursor: "pointer", fontSize: "13px",
            }}>
              {listing.price === 0 ? "免费安装" : "Purchase"}
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}
