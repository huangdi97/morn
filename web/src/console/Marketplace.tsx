import { useState, useEffect, useRef } from "react";
import { api } from "../api";
import { useTranslation } from '../i18n';

interface RatingDistribution {
  1: number;
  2: number;
  3: number;
  4: number;
  5: number;
}

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
  version: string;
  size: string;
  license: string;
  updatedAt: number;
  ratingDistribution: RatingDistribution;
}

const hardcodedListings: Listing[] = [
  { id: "l1", name: "Web Search Pro", item_type: "tool", description: "Advanced web search engine integration with multi-source aggregation", price: 0.001, author: "Morn Labs", rating: 4.5, reviewCount: 42, downloads: 1230, version: "2.1.0", size: "1.2 MB", license: "MIT", updatedAt: Date.now() - 2 * 86400000, ratingDistribution: { 1: 2, 2: 3, 3: 10, 4: 15, 5: 12 } },
  { id: "l2", name: "Stock Market Data", item_type: "knowledge", description: "Real-time stock quotes and financial market data feed", price: 0.01, author: "Morn Labs", rating: 4.2, reviewCount: 28, downloads: 890, version: "1.5.0", size: "890 KB", license: "MIT", updatedAt: Date.now() - 5 * 86400000, ratingDistribution: { 1: 1, 2: 4, 3: 8, 4: 10, 5: 5 } },
  { id: "l3", name: "Deep Research Skill", item_type: "skill", description: "Multi-step research automation with source citation", price: 0.01, author: "Morn Labs", rating: 4.8, reviewCount: 15, downloads: 560, version: "1.0.0", size: "450 KB", license: "Apache-2.0", updatedAt: Date.now() - 1 * 86400000, ratingDistribution: { 1: 0, 2: 1, 3: 2, 4: 3, 5: 9 } },
  { id: "l4", name: "Research Agent", item_type: "agent", description: "Full-featured research agent with web crawling and summarization", price: 0.05, author: "Morn Labs", rating: 4.6, reviewCount: 33, downloads: 340, version: "3.0.0", size: "2.4 MB", license: "MIT", updatedAt: Date.now() - 7 * 86400000, ratingDistribution: { 1: 1, 2: 2, 3: 5, 4: 10, 5: 15 } },
  { id: "l5", name: "Weekly Report", item_type: "workflow", description: "Auto report generation and email delivery workflow", price: 0.03, author: "Morn Labs", rating: 4.1, reviewCount: 8, downloads: 120, version: "1.2.0", size: "320 KB", license: "MIT", updatedAt: Date.now() - 14 * 86400000, ratingDistribution: { 1: 0, 2: 2, 3: 3, 4: 2, 5: 1 } },
  { id: "l6", name: "SQL Expert", item_type: "personality", description: "Expert database query assistant persona", price: 0, author: "Community", rating: 4.3, reviewCount: 19, downloads: 670, version: "1.0.0", size: "180 KB", license: "CC-BY-4.0", updatedAt: Date.now() - 3 * 86400000, ratingDistribution: { 1: 1, 2: 2, 3: 4, 4: 7, 5: 5 } },
  { id: "l7", name: "Memory Optimizer", item_type: "memory", description: "Intelligent memory compression and retrieval optimization", price: 0, author: "Community", rating: 4.0, reviewCount: 12, downloads: 430, version: "0.9.0", size: "560 KB", license: "MIT", updatedAt: Date.now() - 10 * 86400000, ratingDistribution: { 1: 1, 2: 2, 3: 3, 4: 4, 5: 2 } },
  { id: "l8", name: "GPT-4o Mini", item_type: "model", description: "Lightweight model connector for cost-effective inference", price: 0.0005, author: "Morn Labs", rating: 4.7, reviewCount: 55, downloads: 2100, version: "2.0.0", size: "0 B", license: "MIT", updatedAt: Date.now() - 1 * 86400000, ratingDistribution: { 1: 0, 2: 1, 3: 4, 4: 12, 5: 38 } },
  { id: "l9", name: "Data Pipeline Team", item_type: "team", description: "Multi-agent team for ETL and data processing workflows", price: 0.2, author: "Morn Labs", rating: 4.4, reviewCount: 7, downloads: 85, version: "1.0.0", size: "1.8 MB", license: "MIT", updatedAt: Date.now() - 20 * 86400000, ratingDistribution: { 1: 0, 2: 1, 3: 2, 4: 2, 5: 2 } },
];

const categories = ["all", "agent", "team", "tool", "skill", "knowledge", "personality", "memory", "model"];

const categoryColors: Record<string, string> = {
  tool: "#3fb950",
  knowledge: "#bc8cff",
  skill: "#d29922",
  agent: "#58a6ff",
  workflow: "#f0883e",
  personality: "#f78166",
  memory: "#79c0ff",
  model: "#ff7b72",
  team: "#7ee787",
};

function renderStars(rating: number): string {
  const full = Math.round(rating);
  return "★".repeat(full) + "☆".repeat(5 - full);
}

function daysAgo(ts: number): number {
  return Math.floor((Date.now() - ts) / 86400000);
}

function useDebounce<T>(value: T, delay: number): T {
  const [debounced, setDebounced] = useState(value);
  useEffect(() => {
    const id = setTimeout(() => setDebounced(value), delay);
    return () => clearTimeout(id);
  }, [value, delay]);
  return debounced;
}

function DetailModal({ listing, allListings, onClose }: { listing: Listing; allListings: Listing[]; onClose: () => void }) {
  const { t } = useTranslation();
  const totalReviews = Object.values(listing.ratingDistribution).reduce((a, b) => a + b, 0);

  const recommendations = allListings
    .filter(l => l.id !== listing.id && (l.item_type === listing.item_type || l.author === listing.author))
    .slice(0, 3);

  return (
    <div style={{
      position: "fixed", inset: 0, zIndex: 1000,
      display: "flex", alignItems: "center", justifyContent: "center",
      background: "rgba(0,0,0,0.6)",
    }} onClick={onClose}>
      <div style={{
        background: "#161b22", border: "1px solid #30363d", borderRadius: "12px",
        padding: "24px", maxWidth: "560px", width: "90%", maxHeight: "85vh", overflow: "auto",
        color: "#e6edf3",
      }} onClick={e => e.stopPropagation()}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start" }}>
          <div>
            <span style={{
              color: categoryColors[listing.item_type] || "#8b949e", fontSize: "11px",
              textTransform: "uppercase", fontWeight: "bold",
            }}>{t(`console.marketplace.category.${listing.item_type}`)}</span>
            <h2 style={{ margin: "4px 0", fontSize: "20px" }}>{listing.name}</h2>
          </div>
          <button onClick={onClose} style={{
            background: "none", border: "none", color: "#8b949e", cursor: "pointer", fontSize: "20px", lineHeight: 1,
          }}>×</button>
        </div>

        <p style={{ color: "#8b949e", fontSize: "14px", margin: "12px 0" }}>{listing.description}</p>

        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "8px", fontSize: "13px", margin: "16px 0" }}>
          <div><span style={{ color: "#8b949e" }}>{t('console.marketplace.detail.version')}:</span> {listing.version}</div>
          <div><span style={{ color: "#8b949e" }}>{t('console.marketplace.detail.size')}:</span> {listing.size}</div>
          <div><span style={{ color: "#8b949e" }}>{t('console.marketplace.detail.license')}:</span> {listing.license}</div>
          <div><span style={{ color: "#8b949e" }}>{t('console.marketplace.detail.author')}:</span> {listing.author}</div>
          <div><span style={{ color: "#8b949e" }}>{t('console.marketplace.detail.updated')}:</span> {t('console.marketplace.days_ago', { count: daysAgo(listing.updatedAt) })}</div>
          <div><span style={{ color: "#8b949e" }}>{t('console.marketplace.detail.downloads')}:</span> {listing.downloads.toLocaleString()}</div>
        </div>

        <div style={{ margin: "16px 0" }}>
          <div style={{ color: "#e6edf3", fontSize: "14px", fontWeight: "bold", marginBottom: "8px" }}>
            {t('console.marketplace.detail.rating_distribution')}
          </div>
          {[5, 4, 3, 2, 1].map(star => {
            const count = listing.ratingDistribution[star as keyof RatingDistribution] || 0;
            const pct = totalReviews > 0 ? Math.round((count / totalReviews) * 100) : 0;
            return (
              <div key={star} style={{ display: "flex", alignItems: "center", gap: "8px", marginBottom: "4px", fontSize: "13px" }}>
                <span style={{ width: "40px", color: "#d29922" }}>{"★".repeat(star)}{"☆".repeat(5 - star)}</span>
                <div style={{ flex: 1, height: "8px", background: "#21262d", borderRadius: "4px", overflow: "hidden" }}>
                  <div style={{ width: `${pct}%`, height: "100%", background: "#d29922", borderRadius: "4px" }} />
                </div>
                <span style={{ width: "36px", textAlign: "right", color: "#8b949e" }}>{pct}%</span>
              </div>
            );
          })}
          <div style={{ color: "#8b949e", fontSize: "12px", marginTop: "4px" }}>
            {t('console.marketplace.detail.total_reviews', { count: listing.reviewCount })}
          </div>
        </div>

        <button style={{
          width: "100%", padding: "10px", marginTop: "8px",
          background: "#1f6feb", color: "#fff", border: "none",
          borderRadius: "6px", cursor: "pointer", fontSize: "14px", fontWeight: "bold",
        }}>
          {listing.price === 0
            ? t('console.marketplace.install_free')
            : `${t('console.marketplace.purchase')} ¥${listing.price.toFixed(3)}`}
        </button>

        {recommendations.length > 0 && (
          <div style={{ marginTop: "20px", borderTop: "1px solid #30363d", paddingTop: "16px" }}>
            <div style={{ color: "#e6edf3", fontSize: "14px", fontWeight: "bold", marginBottom: "8px" }}>
              {t('console.marketplace.detail.you_may_also_like')}
            </div>
            <div style={{ display: "flex", gap: "8px", flexWrap: "wrap" }}>
              {recommendations.map(r => (
                <div key={r.id} style={{
                  background: "#0d1117", border: "1px solid #30363d", borderRadius: "6px",
                  padding: "8px 12px", flex: "1", minWidth: "120px", cursor: "pointer",
                }}>
                  <div style={{ color: "#e6edf3", fontSize: "13px", fontWeight: "bold" }}>{r.name}</div>
                  <div style={{ color: "#d29922", fontSize: "11px" }}>{renderStars(r.rating)}</div>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

export default function Marketplace() {
  const { t } = useTranslation();
  const [listings, setListings] = useState<Listing[]>(hardcodedListings);
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedCategory, setSelectedCategory] = useState("all");
  const [detailListing, setDetailListing] = useState<Listing | null>(null);
  const [showDistribution, setShowDistribution] = useState<string | null>(null);
  const searchRef = useRef<HTMLInputElement>(null);
  const debouncedQuery = useDebounce(searchQuery, 300);

  useEffect(() => {
    api.getMarketListings(null).then((data: Listing[]) => {
      if (Array.isArray(data)) setListings(data);
    }).catch(() => {
      setListings(hardcodedListings);
    });
  }, []);

  const filtered = listings.filter(item => {
    const matchesCategory = selectedCategory === "all" || item.item_type === selectedCategory;
    if (!debouncedQuery) return matchesCategory;
    const q = debouncedQuery.toLowerCase();
    const matchesSearch = item.name.toLowerCase().includes(q) || item.description.toLowerCase().includes(q);
    return matchesCategory && matchesSearch;
  });

  return (
    <div>
      <h2 style={{ color: "#e6edf3", marginBottom: "16px" }}>{t('console.marketplace.title')}</h2>

      <div style={{ marginBottom: "12px" }}>
        <input
          ref={searchRef}
          type="text"
          value={searchQuery}
          onChange={e => setSearchQuery(e.target.value)}
          placeholder={t('console.marketplace.search_placeholder')}
          style={{
            width: "100%", padding: "8px 12px", boxSizing: "border-box",
            background: "#0d1117", border: "1px solid #30363d", borderRadius: "6px",
            color: "#e6edf3", fontSize: "14px", outline: "none",
          }}
        />
      </div>

      {debouncedQuery && (
        <div style={{ color: "#8b949e", fontSize: "13px", marginBottom: "12px" }}>
          {t('console.marketplace.search_results', { count: filtered.length })}
        </div>
      )}

      <div style={{ display: "flex", gap: "6px", marginBottom: "16px", flexWrap: "wrap" }}>
        {categories.map(cat => (
          <button key={cat} onClick={() => setSelectedCategory(cat)}
            style={{
              background: selectedCategory === cat ? "#1f6feb" : "transparent",
              color: selectedCategory === cat ? "#fff" : "#8b949e",
              border: `1px solid ${selectedCategory === cat ? "#1f6feb" : "#30363d"}`,
              padding: "4px 12px", borderRadius: "4px", cursor: "pointer",
              fontSize: "13px",
            }}>
            {t(`console.marketplace.category.${cat}`)}
          </button>
        ))}
      </div>

      <div style={{ display: "grid", gridTemplateColumns: "repeat(2, 1fr)", gap: "12px" }}>
        {filtered.map(listing => (
          <div key={listing.id} className="market-card"
            style={{ cursor: "pointer" }}
            onClick={() => setDetailListing(listing)}>
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
              <span style={{
                color: categoryColors[listing.item_type] || "#8b949e", fontSize: "11px",
                textTransform: "uppercase", fontWeight: "bold",
              }}>
                {t(`console.marketplace.category.${listing.item_type}`)}
              </span>
              <div style={{ display: "flex", alignItems: "center", gap: "4px", position: "relative" }}>
                <span style={{ color: "#d29922", fontSize: "13px", cursor: "pointer" }}
                  onClick={e => { e.stopPropagation(); setShowDistribution(showDistribution === listing.id ? null : listing.id); }}>
                  {renderStars(listing.rating)}
                </span>
                <span style={{ color: "#8b949e", fontSize: "11px" }}>({listing.reviewCount})</span>
                {showDistribution === listing.id && (
                  <div style={{
                    position: "absolute", top: "20px", right: 0, zIndex: 100,
                    background: "#161b22", border: "1px solid #30363d", borderRadius: "8px",
                    padding: "12px", minWidth: "200px",
                  }} onClick={e => e.stopPropagation()}>
                    <div style={{ color: "#e6edf3", fontSize: "12px", fontWeight: "bold", marginBottom: "8px" }}>
                      {t('console.marketplace.detail.rating_distribution')}
                    </div>
                    {[5, 4, 3, 2, 1].map(star => {
                      const count = listing.ratingDistribution[star as keyof RatingDistribution] || 0;
                      const total = Object.values(listing.ratingDistribution).reduce((a, b) => a + b, 0);
                      const pct = total > 0 ? Math.round((count / total) * 100) : 0;
                      return (
                        <div key={star} style={{ display: "flex", alignItems: "center", gap: "6px", marginBottom: "3px", fontSize: "12px" }}>
                          <span style={{ width: "36px", color: "#d29922" }}>{"★".repeat(star)}{"☆".repeat(5 - star)}</span>
                          <div style={{ flex: 1, height: "6px", background: "#21262d", borderRadius: "3px", overflow: "hidden" }}>
                            <div style={{ width: `${pct}%`, height: "100%", background: "#d29922", borderRadius: "3px" }} />
                          </div>
                          <span style={{ width: "30px", textAlign: "right", color: "#8b949e" }}>{pct}%</span>
                        </div>
                      );
                    })}
                  </div>
                )}
              </div>
            </div>
            <div style={{ color: "#e6edf3", fontWeight: "bold", marginTop: "8px" }}>{listing.name}</div>
            <div style={{ color: "#8b949e", fontSize: "13px", marginTop: "4px", lineHeight: 1.4 }}>{listing.description}</div>
            <div style={{ display: "flex", justifyContent: "space-between", marginTop: "12px" }}>
              <span style={{ color: "#8b949e", fontSize: "12px" }}>
                {t('console.marketplace.by_author', { author: listing.author })} · {listing.downloads.toLocaleString()} {t('console.marketplace.downloads')} · {t('console.marketplace.days_ago', { count: daysAgo(listing.updatedAt) })}
              </span>
              {listing.price === 0 ? (
                <span style={{ color: "#3fb950", fontSize: "11px", fontWeight: "bold", background: "rgba(63,185,80,0.15)", padding: "2px 8px", borderRadius: "4px" }}>
                  {t('console.marketplace.free')}
                </span>
              ) : (
                <span style={{ color: "#f85149", fontWeight: "bold", fontSize: "13px" }}>¥{listing.price.toFixed(3)}</span>
              )}
            </div>
            <button style={{
              width: "100%", marginTop: "12px", padding: "6px",
              background: "#1f6feb", color: "#fff", border: "none",
              borderRadius: "4px", cursor: "pointer", fontSize: "13px",
            }} onClick={e => {
              e.stopPropagation();
              setDetailListing(listing);
            }}>
              {listing.price === 0 ? t('console.marketplace.install_free') : t('console.marketplace.purchase')}
            </button>
          </div>
        ))}
      </div>

      {filtered.length === 0 && (
        <div style={{ color: "#8b949e", textAlign: "center", padding: "40px", fontSize: "14px" }}>
          {t('console.marketplace.no_results')}
        </div>
      )}

      {detailListing && (
        <DetailModal listing={detailListing} allListings={listings} onClose={() => setDetailListing(null)} />
      )}
    </div>
  );
}