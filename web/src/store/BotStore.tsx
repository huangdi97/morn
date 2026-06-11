import { useState, useEffect } from "react";
import { api } from "../api";

interface BotListing {
  id: string;
  name: string;
  icon: string;
  description: string;
  category: string;
  rating: number;
  installs: number;
  author: string;
  price: number;
  template_id: string;
}

const hardcodedBots: BotListing[] = [
  { id: "b1", name: "Data Analyst", icon: "📊", description: "Turn raw data into actionable insights with statistical analysis and visualization", category: "analysis", rating: 4.8, installs: 3420, author: "Morn Labs", price: 0, template_id: "preset-analyst" },
  { id: "b2", name: "Research Assistant", icon: "🔬", description: "Multi-source research with cross-validation and citation management", category: "research", rating: 4.7, installs: 2890, author: "Morn Labs", price: 0, template_id: "preset-researcher" },
  { id: "b3", name: "Content Writer", icon: "✍️", description: "Create engaging content from blog posts to technical documentation", category: "writing", rating: 4.6, installs: 2150, author: "Morn Labs", price: 0, template_id: "preset-writer" },
  { id: "b4", name: "Code Engineer", icon: "💻", description: "Full-stack development with testing and best practices", category: "coding", rating: 4.9, installs: 4560, author: "Morn Labs", price: 0, template_id: "preset-coder" },
  { id: "b5", name: "Translator Pro", icon: "🌐", description: "Professional translation with cultural adaptation and terminology management", category: "translation", rating: 4.5, installs: 1870, author: "Morn Labs", price: 0.001, template_id: "preset-translator" },
  { id: "b6", name: "System Assistant", icon: "🤖", description: "All-purpose AI assistant for daily tasks and workflow automation", category: "assistant", rating: 4.4, installs: 5230, author: "Morn Labs", price: 0, template_id: "preset-assistant" },
  { id: "b7", name: "Code Reviewer", icon: "🔍", description: "Thorough code review with actionable improvement suggestions", category: "review", rating: 4.7, installs: 1560, author: "Morn Labs", price: 0, template_id: "preset-reviewer" },
  { id: "b8", name: "Customer Support", icon: "🎧", description: "Patient and empathetic customer service agent", category: "support", rating: 4.3, installs: 980, author: "Morn Labs", price: 0, template_id: "preset-cs-agent" },
  { id: "b9", name: "Financial Analyst", icon: "💰", description: "Financial data analysis, trend prediction and investment research", category: "analysis", rating: 4.6, installs: 1340, author: "Morn Labs", price: 0.002, template_id: "preset-analyst" },
  { id: "b10", name: "DevOps Bot", icon: "⚙️", description: "Infrastructure management, deployment automation and monitoring", category: "coding", rating: 4.5, installs: 870, author: "Morn Labs", price: 0, template_id: "preset-coder" },
];

const categories = ["all", "analysis", "research", "writing", "coding", "translation", "assistant", "review", "support"];

export default function BotStore() {
  const [bots, setBots] = useState<BotListing[]>(hardcodedBots);
  const [category, setCategory] = useState("all");
  const [installed, setInstalled] = useState<Set<string>>(new Set());
  const [search, setSearch] = useState("");

  useEffect(() => {
    api.listBotStore().then(setBots).catch(() => {
      setBots(hardcodedBots);
    });
  }, []);

  const filtered = bots.filter(b => {
    const matchCategory = category === "all" || b.category === category;
    const matchSearch = !search || b.name.toLowerCase().includes(search.toLowerCase()) || b.description.toLowerCase().includes(search.toLowerCase());
    return matchCategory && matchSearch;
  });

  const handleInstall = (bot: BotListing) => {
    if (installed.has(bot.id)) return;
    api.installBotFromStore(bot.id, bot.template_id)
      .then(() => setInstalled(prev => new Set(prev).add(bot.id)))
      .catch(console.error);
  };

  const getCategoryColor = (cat: string) => {
    const colors: Record<string, string> = {
      analysis: "#3fb950", research: "#bc8cff", writing: "#d29922",
      coding: "#58a6ff", translation: "#f0883e", assistant: "#8b949e",
      review: "#f85149", support: "#db6d28",
    };
    return colors[cat] || "#8b949e";
  };

  return (
    <div>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "16px" }}>
        <h2 style={{ color: "#e6edf3", margin: 0 }}>Bot Store</h2>
        <input
          type="text" placeholder="Search bots..."
          value={search} onChange={e => setSearch(e.target.value)}
          style={{
            background: "#0d1117", border: "1px solid #30363d", borderRadius: "4px",
            padding: "6px 12px", color: "#e6edf3", fontSize: "13px", width: "240px",
          }}
        />
      </div>

      <div style={{ display: "flex", gap: "8px", marginBottom: "16px", flexWrap: "wrap" }}>
        {categories.map(c => (
          <button key={c} onClick={() => setCategory(c)}
            style={{
              background: category === c ? "#1f6feb" : "transparent",
              color: category === c ? "#fff" : "#8b949e",
              border: "1px solid #30363d", padding: "4px 12px",
              borderRadius: "4px", cursor: "pointer", fontSize: "13px",
              textTransform: "capitalize",
            }}>
            {c}
          </button>
        ))}
      </div>

      <div style={{ display: "grid", gridTemplateColumns: "repeat(3, 1fr)", gap: "12px" }}>
        {filtered.map(bot => (
          <div key={bot.id} className="market-card" style={{ display: "flex", flexDirection: "column" }}>
            <div style={{ display: "flex", alignItems: "center", gap: "10px", marginBottom: "8px" }}>
              <span style={{ fontSize: "28px" }}>{bot.icon}</span>
              <div>
                <div style={{ color: "#e6edf3", fontWeight: "bold" }}>{bot.name}</div>
                <span style={{ color: getCategoryColor(bot.category), fontSize: "11px", textTransform: "uppercase" }}>
                  {bot.category}
                </span>
              </div>
              <span style={{ marginLeft: "auto", color: "#d29922", fontSize: "13px" }}>★ {bot.rating.toFixed(1)}</span>
            </div>
            <div style={{ color: "#8b949e", fontSize: "13px", flex: 1 }}>{bot.description}</div>
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginTop: "12px" }}>
              <span style={{ color: "#8b949e", fontSize: "12px" }}>
                {bot.author} · {bot.installs} installs
              </span>
              <span style={{ color: "#f85149", fontSize: "13px", fontWeight: "bold" }}>
                {bot.price === 0 ? "FREE" : `¥${bot.price.toFixed(3)}`}
              </span>
            </div>
            <button
              onClick={() => handleInstall(bot)}
              disabled={installed.has(bot.id)}
              style={{
                width: "100%", marginTop: "12px", padding: "6px",
                background: installed.has(bot.id) ? "#21262d" : "#1f6feb",
                color: installed.has(bot.id) ? "#8b949e" : "#fff",
                border: "none", borderRadius: "4px", cursor: installed.has(bot.id) ? "default" : "pointer",
                fontSize: "13px",
              }}>
              {installed.has(bot.id) ? "Installed ✓" : bot.price === 0 ? "Install" : "Purchase"}
            </button>
          </div>
        ))}
      </div>

      {filtered.length === 0 && (
        <div style={{ color: "#8b949e", textAlign: "center", padding: "40px" }}>
          No bots found matching your criteria
        </div>
      )}
    </div>
  );
}
