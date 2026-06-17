import { useState, useEffect } from "react";
import { api } from "../api";
import CheckoutModal from "./CheckoutModal";

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

interface Review {
  id: string;
  listing_id: string;
  user_id: string;
  rating: number;
  comment: string;
  created_at: string;
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
const priceFilters = ["all", "free", "paid"] as const;

export default function BotStore() {
  const [loading, setLoading] = useState(true);
  const [bots, setBots] = useState<BotListing[]>(hardcodedBots);
  const [category, setCategory] = useState("all");
  const [priceFilter, setPriceFilter] = useState<string>("all");
  const [installed, setInstalled] = useState<Set<string>>(new Set());
  const [search, setSearch] = useState("");
  const [reviews, setReviews] = useState<Record<string, Review[]>>({});
  const [reviewForm, setReviewForm] = useState<Record<string, { rating: number; comment: string }>>({});
  const [expandedReviews, setExpandedReviews] = useState<Set<string>>(new Set());
  const [checkoutBot, setCheckoutBot] = useState<BotListing | null>(null);
  const [publishStatus, setPublishStatus] = useState<Record<string, string>>({});

  useEffect(() => {
    api.listBotStore().then((res) => {
      setBots(res);
      setLoading(false);
    }).catch(() => {
      setBots(hardcodedBots);
      setLoading(false);
    });
  }, []);

  const filtered = bots.filter(b => {
    const matchCategory = category === "all" || b.category === category;
    const matchPrice = priceFilter === "all" || (priceFilter === "free" && b.price === 0) || (priceFilter === "paid" && b.price > 0);
    const matchSearch = !search || b.name.toLowerCase().includes(search.toLowerCase()) || b.description.toLowerCase().includes(search.toLowerCase());
    return matchCategory && matchPrice && matchSearch;
  });

  const handleInstall = (bot: BotListing) => {
    if (installed.has(bot.id)) return;
    if (bot.price > 0) {
      setCheckoutBot(bot);
      return;
    }
    doInstall(bot);
  };

  const doInstall = (bot: BotListing) => {
    api.installBotFromStore(bot.id, bot.template_id)
      .then(() => setInstalled(prev => new Set(prev).add(bot.id)))
      .catch(console.error);
  };

  const handlePublish = (bot: BotListing) => {
    const categoryMap: Record<string, string> = {
      analysis: "data-analysis", research: "research", writing: "writing",
      coding: "coding", translation: "translation", assistant: "assistant",
      review: "code-review", support: "customer-support",
    };
    setPublishStatus(prev => ({ ...prev, [bot.id]: "publishing..." }));
    api.hubPublish({
      name: bot.name,
      description: bot.description,
      category: categoryMap[bot.category] || bot.category,
      version: "1.0.0",
      screenshots: "",
      price: 0,
      author: "Morn Labs",
      itemType: "agent",
    }).then(() => {
      setPublishStatus(prev => ({ ...prev, [bot.id]: "Published!" }));
      setTimeout(() => setPublishStatus(prev => ({ ...prev, [bot.id]: "" })), 2000);
    }).catch((err: Error) => {
      setPublishStatus(prev => ({ ...prev, [bot.id]: err.message || "Error" }));
    });
  };

  const handlePurchaseConfirm = async () => {
    if (!checkoutBot) return;
    await api.installBotFromStore(checkoutBot.id, checkoutBot.template_id);
    setInstalled(prev => new Set(prev).add(checkoutBot.id));
  };

  const handleSearch = (val: string) => {
    setSearch(val);
    api.searchMarketListings(val || null, category === "all" ? null : category).then(setBots).catch(() => {});
  };

  const handleCategoryChange = (cat: string) => {
    setCategory(cat);
    api.searchMarketListings(search || null, cat === "all" ? null : cat).then(setBots).catch(() => {});
  };

  const handlePriceFilterChange = (pf: string) => {
    setPriceFilter(pf);
  };

  const toggleReviews = (botId: string) => {
    const next = new Set(expandedReviews);
    if (next.has(botId)) {
      next.delete(botId);
    } else {
      next.add(botId);
      api.getListingReviews(botId).then((data: Review[]) => {
        setReviews(prev => ({ ...prev, [botId]: data }));
      }).catch(() => {});
    }
    setExpandedReviews(next);
  };

  const handleSubmitReview = (botId: string) => {
    const form = reviewForm[botId];
    if (!form || form.rating < 1 || form.rating > 5 || !form.comment.trim()) return;
    api.submitReview(botId, form.rating, form.comment).then(() => {
      api.getListingReviews(botId).then((data: Review[]) => {
        setReviews(prev => ({ ...prev, [botId]: data }));
      });
      setReviewForm(prev => ({ ...prev, [botId]: { rating: 5, comment: "" } }));
    }).catch(console.error);
  };

  const getCategoryColor = (cat: string) => {
    const colors: Record<string, string> = {
      analysis: "#3fb950", research: "#bc8cff", writing: "#d29922",
      coding: "#58a6ff", translation: "#f0883e", assistant: "#8b949e",
      review: "#f85149", support: "#db6d28",
    };
    return colors[cat] || "#8b949e";
  };

  if (loading) {
    return <div style={{ padding: '20px', textAlign: 'center', color: 'var(--text-secondary)' }}>正在加载商店…</div>;
  }

  return (
    <div>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "16px" }}>
        <h2 style={{ color: "#e6edf3", margin: 0 }}>Bot Store</h2>
        <input
          type="text" placeholder="Search bots..."
          value={search} onChange={e => handleSearch(e.target.value)}
          style={{
            background: "#0d1117", border: "1px solid #30363d", borderRadius: "4px",
            padding: "6px 12px", color: "#e6edf3", fontSize: "13px", width: "240px",
          }}
        />
      </div>

      <div style={{ marginBottom: "16px", display: "flex", gap: "8px", alignItems: "center", flexWrap: "wrap" }}>
        <span style={{ color: "#8b949e", fontSize: "12px" }}>Price:</span>
        {priceFilters.map(pf => (
          <button key={pf} onClick={() => handlePriceFilterChange(pf)}
            style={{
              background: priceFilter === pf ? "#1f6feb" : "transparent",
              color: priceFilter === pf ? "#fff" : "#8b949e",
              border: "1px solid #30363d", borderRadius: "4px",
              padding: "4px 10px", cursor: "pointer", fontSize: "12px",
              textTransform: "capitalize",
            }}>
            {pf === "all" ? "All" : pf === "free" ? "免费" : "付费"}
          </button>
        ))}
        <span style={{ color: "#30363d" }}>|</span>
        <span style={{ color: "#8b949e", fontSize: "12px" }}>Category:</span>
        {categories.map(c => (
          <button key={c} onClick={() => handleCategoryChange(c)}
            style={{
              background: category === c ? "#1f6feb" : "transparent",
              color: category === c ? "#fff" : "#8b949e",
              border: "1px solid #30363d", borderRadius: "4px",
              padding: "4px 10px", cursor: "pointer", fontSize: "12px",
              textTransform: "capitalize",
            }}>
            {c === "all" ? "All" : c}
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
            <div style={{ display: "flex", gap: "6px" }}>
              <button
                onClick={() => handleInstall(bot)}
                disabled={installed.has(bot.id)}
                style={{
                  flex: 1, marginTop: "12px", padding: "6px",
                  background: installed.has(bot.id) ? "#21262d" : "#1f6feb",
                  color: installed.has(bot.id) ? "#8b949e" : "#fff",
                  border: "none", borderRadius: "4px", cursor: installed.has(bot.id) ? "default" : "pointer",
                  fontSize: "13px",
                }}>
                {installed.has(bot.id) ? "Installed ✓" : bot.price === 0 ? "免费安装" : `购买 ¥${bot.price.toFixed(3)}`}
              </button>
              <button
                onClick={() => handlePublish(bot)}
                style={{
                  marginTop: "12px", padding: "6px 10px",
                  background: "transparent", color: "#3fb950",
                  border: "1px solid #3fb950", borderRadius: "4px", cursor: "pointer", fontSize: "13px", whiteSpace: "nowrap",
                }}>
                {publishStatus[bot.id] || "Publish"}
              </button>
            </div>
            <button
              onClick={() => toggleReviews(bot.id)}
              style={{
                width: "100%", marginTop: "6px", padding: "6px",
                background: "transparent", color: "#58a6ff",
                border: "1px solid #30363d", borderRadius: "4px", cursor: "pointer", fontSize: "13px",
              }}>
              {expandedReviews.has(bot.id) ? "Hide Reviews" : "Show Reviews"}
            </button>
            {expandedReviews.has(bot.id) && (
              <div style={{ marginTop: "8px", borderTop: "1px solid #30363d", paddingTop: "8px" }}>
                {reviews[bot.id]?.map(r => (
                  <div key={r.id} style={{ fontSize: "12px", color: "#8b949e", marginBottom: "6px", padding: "4px 0", borderBottom: "1px solid #21262d" }}>
                    <span style={{ color: "#d29922" }}>{"★".repeat(r.rating)}{"☆".repeat(5 - r.rating)}</span>
                    <span> {r.comment}</span>
                  </div>
                )) || <div style={{ fontSize: "12px", color: "#8b949e", marginBottom: "6px" }}>No reviews yet</div>}
                <div style={{ display: "flex", gap: "6px", marginTop: "6px", alignItems: "center" }}>
                  <select value={reviewForm[bot.id]?.rating || 5} onChange={e => setReviewForm(prev => ({ ...prev, [bot.id]: { rating: Number(e.target.value), comment: prev[bot.id]?.comment || "" } }))}
                    style={{ background: "#0d1117", border: "1px solid #30363d", borderRadius: "4px", padding: "4px", color: "#e6edf3", fontSize: "12px" }}>
                    {[5, 4, 3, 2, 1].map(n => <option key={n} value={n}>{n}★</option>)}
                  </select>
                  <input type="text" placeholder="Write a review..." value={reviewForm[bot.id]?.comment || ""}
                    onChange={e => setReviewForm(prev => ({ ...prev, [bot.id]: { rating: prev[bot.id]?.rating || 5, comment: e.target.value } }))}
                    style={{ flex: 1, background: "#0d1117", border: "1px solid #30363d", borderRadius: "4px", padding: "4px 8px", color: "#e6edf3", fontSize: "12px" }} />
                  <button onClick={() => handleSubmitReview(bot.id)}
                    style={{ background: "#1f6feb", color: "#fff", border: "none", borderRadius: "4px", padding: "4px 8px", cursor: "pointer", fontSize: "12px" }}>
                    Submit
                  </button>
                </div>
              </div>
            )}
          </div>
        ))}
      </div>

      {filtered.length === 0 && (
        <div style={{ color: "#8b949e", textAlign: "center", padding: "40px" }}>
          没有找到匹配的 Bot
        </div>
      )}

      {checkoutBot && (
        <CheckoutModal
          name={checkoutBot.name}
          icon={checkoutBot.icon}
          price={checkoutBot.price}
          onConfirm={handlePurchaseConfirm}
          onClose={() => { setCheckoutBot(null); }}
        />
      )}
    </div>
  );
}