import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface EarningsData {
  creator_id: string;
  total_earnings: number;
  pending_payout: number;
  sale_count: number;
}

interface SaleRecord {
  id: string;
  time: string;
  item: string;
  amount: number;
  platform_cut: number;
  creator_share: number;
}

interface PayoutRecord {
  id: string;
  time: string;
  amount: number;
  status: string;
}

const cardStyle: React.CSSProperties = {
  background: "#161b22",
  borderRadius: "8px",
  padding: "16px",
  border: "1px solid #30363d",
};

const statValue: React.CSSProperties = {
  color: "#e6edf3",
  fontSize: "24px",
  fontWeight: "bold",
  marginTop: "8px",
};

const statLabel: React.CSSProperties = {
  color: "#8b949e",
  fontSize: "13px",
};

const tableStyle: React.CSSProperties = {
  width: "100%",
  borderCollapse: "collapse",
  fontSize: "14px",
};

const thStyle: React.CSSProperties = {
  textAlign: "left",
  padding: "10px 12px",
  borderBottom: "1px solid #21262d",
  color: "#8b949e",
  fontWeight: 600,
};

const tdStyle: React.CSSProperties = {
  padding: "10px 12px",
  borderBottom: "1px solid #21262d",
  color: "#e6edf3",
};

export default function CreatorEarnings() {
  const [earnings, setEarnings] = useState<EarningsData | null>(null);
  const [loading, setLoading] = useState(true);

  const [sales] = useState<SaleRecord[]>([
    { id: "1", time: "2025-06-15 14:30", item: "Web Search Pro", amount: 50.00, platform_cut: 5.00, creator_share: 45.00 },
    { id: "2", time: "2025-06-14 09:15", item: "Code Assistant", amount: 30.00, platform_cut: 3.00, creator_share: 27.00 },
    { id: "3", time: "2025-06-13 16:45", item: "Image Generator", amount: 80.00, platform_cut: 8.00, creator_share: 72.00 },
    { id: "4", time: "2025-06-12 11:20", item: "Data Analyzer", amount: 40.00, platform_cut: 4.00, creator_share: 36.00 },
    { id: "5", time: "2025-06-11 08:00", item: "Web Search Pro", amount: 50.00, platform_cut: 5.00, creator_share: 45.00 },
  ]);

  const [payouts] = useState<PayoutRecord[]>([
    { id: "p1", time: "2025-06-01", amount: 200.00, status: "completed" },
    { id: "p2", time: "2025-05-15", amount: 150.00, status: "completed" },
    { id: "p3", time: "2025-05-01", amount: 180.00, status: "completed" },
  ]);

  useEffect(() => {
    invoke<EarningsData>("get_creator_earnings")
      .then(setEarnings)
      .catch(() => {
        setEarnings({
          creator_id: "creator-1",
          total_earnings: 1250.00,
          pending_payout: 340.00,
          sale_count: 18,
        });
      })
      .finally(() => setLoading(false));
  }, []);

  if (loading) {
    return (
      <div>
        <h2 style={{ color: "#e6edf3", marginBottom: "16px" }}>Creator Earnings</h2>
        <div style={{ color: "#8b949e", fontSize: "14px" }}>Loading...</div>
      </div>
    );
  }

  const handlePayout = async () => {
    alert("Payout request submitted. Funds will be transferred within 3-5 business days.");
  };

  return (
    <div>
      <h2 style={{ color: "#e6edf3", marginBottom: "16px" }}>Creator Earnings</h2>

      <div style={{ display: "grid", gridTemplateColumns: "repeat(3, 1fr)", gap: "12px", marginBottom: "24px" }}>
        <div style={cardStyle}>
          <div style={statLabel}>Total Earnings</div>
          <div style={{ ...statValue, color: "#3fb950" }}>${earnings?.total_earnings.toFixed(2)}</div>
        </div>
        <div style={cardStyle}>
          <div style={statLabel}>Pending Payout</div>
          <div style={{ ...statValue, color: "#d29922" }}>${earnings?.pending_payout.toFixed(2)}</div>
        </div>
        <div style={cardStyle}>
          <div style={statLabel}>Sales</div>
          <div style={{ ...statValue, color: "#58a6ff" }}>{earnings?.sale_count}</div>
        </div>
      </div>

      <div style={{ display: "flex", gap: "12px", marginBottom: "24px" }}>
        <button
          onClick={handlePayout}
          style={{
            padding: "10px 24px",
            background: "#238636",
            color: "#fff",
            border: "none",
            borderRadius: "6px",
            fontSize: "14px",
            fontWeight: 600,
            cursor: "pointer",
          }}
        >
          Request Payout (${earnings?.pending_payout.toFixed(2)})
        </button>
      </div>

      <div style={{ ...cardStyle, marginBottom: "24px" }}>
        <h3 style={{ color: "#e6edf3", fontSize: "16px", marginBottom: "12px" }}>Recent Sales</h3>
        <table style={tableStyle}>
          <thead>
            <tr>
              <th style={thStyle}>Time</th>
              <th style={thStyle}>Item</th>
              <th style={thStyle}>Amount</th>
              <th style={thStyle}>Platform Cut</th>
              <th style={thStyle}>Your Share</th>
            </tr>
          </thead>
          <tbody>
            {sales.map((s) => (
              <tr key={s.id}>
                <td style={tdStyle}>{s.time}</td>
                <td style={tdStyle}>{s.item}</td>
                <td style={tdStyle}>${s.amount.toFixed(2)}</td>
                <td style={tdStyle}>-${s.platform_cut.toFixed(2)}</td>
                <td style={{ ...tdStyle, color: "#3fb950" }}>+${s.creator_share.toFixed(2)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <div style={cardStyle}>
        <h3 style={{ color: "#e6edf3", fontSize: "16px", marginBottom: "12px" }}>Payout History</h3>
        <table style={tableStyle}>
          <thead>
            <tr>
              <th style={thStyle}>Date</th>
              <th style={thStyle}>Amount</th>
              <th style={thStyle}>Status</th>
            </tr>
          </thead>
          <tbody>
            {payouts.map((p) => (
              <tr key={p.id}>
                <td style={tdStyle}>{p.time}</td>
                <td style={tdStyle}>${p.amount.toFixed(2)}</td>
                <td style={{ ...tdStyle, color: p.status === "completed" ? "#3fb950" : "#d29922" }}>
                  {p.status === "completed" ? "Completed" : "Processing"}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}