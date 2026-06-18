import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from '../i18n';

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
  const { t } = useTranslation();
  const [earnings, setEarnings] = useState<EarningsData | null>(null);
  const [sales, setSales] = useState<SaleRecord[]>([]);
  const [payouts, setPayouts] = useState<PayoutRecord[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    Promise.all([
      invoke<EarningsData>("get_creator_earnings"),
      invoke<SaleRecord[]>("get_earnings_history"),
      invoke<PayoutRecord[]>("get_payment_history"),
    ])
      .then(([earningsData, salesData, payoutsData]) => {
        setEarnings(earningsData);
        setSales(salesData);
        setPayouts(payoutsData);
      })
      .catch(() => {})
      .finally(() => setLoading(false));
  }, []);

  if (loading) {
    return (
      <div>
        <h2 style={{ color: "#e6edf3", marginBottom: "16px" }}>{t('console.earnings.title')}</h2>
        <div style={{ color: "#8b949e", fontSize: "14px" }}>{t('console.earnings.loading')}</div>
      </div>
    );
  }

  const handlePayout = async () => {
    alert("Payout request submitted. Funds will be transferred within 3-5 business days.");
  };

  if (!earnings) {
    return (
      <div>
        <h2 style={{ color: "#e6edf3", marginBottom: "16px" }}>{t('console.earnings.title')}</h2>
        <div style={{ color: "#8b949e", fontSize: "14px" }}>{t('console.earnings.loading')}</div>
      </div>
    );
  }

  return (
    <div>
      <h2 style={{ color: "#e6edf3", marginBottom: "16px" }}>{t('console.earnings.title')}</h2>

      <div style={{ display: "grid", gridTemplateColumns: "repeat(3, 1fr)", gap: "12px", marginBottom: "24px" }}>
        <div style={cardStyle}>
          <div style={statLabel}>{t('console.earnings.total_earnings')}</div>
          <div style={{ ...statValue, color: "#3fb950" }}>${earnings.total_earnings.toFixed(2)}</div>
        </div>
        <div style={cardStyle}>
          <div style={statLabel}>Pending Payout</div>
          <div style={{ ...statValue, color: "#d29922" }}>${earnings.pending_payout.toFixed(2)}</div>
        </div>
        <div style={cardStyle}>
          <div style={statLabel}>Sales</div>
          <div style={{ ...statValue, color: "#58a6ff" }}>{earnings.sale_count}</div>
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
          {t('console.earnings.request_payout')} (${earnings.pending_payout.toFixed(2)})
        </button>
      </div>

      <div style={{ ...cardStyle, marginBottom: "24px" }}>
        <h3 style={{ color: "#e6edf3", fontSize: "16px", marginBottom: "12px" }}>{t('console.earnings.recent_sales')}</h3>
        {sales.length === 0 ? (
          <div style={{ color: "#8b949e", fontSize: "14px" }}>暂无销售记录</div>
        ) : (
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
        )}
      </div>

      <div style={cardStyle}>
        <h3 style={{ color: "#e6edf3", fontSize: "16px", marginBottom: "12px" }}>{t('console.earnings.payout_history')}</h3>
        {payouts.length === 0 ? (
          <div style={{ color: "#8b949e", fontSize: "14px" }}>暂无提现记录</div>
        ) : (
          <table style={tableStyle}>
            <thead>
              <tr>
                <th style={thStyle}>Date</th>
                <th style={thStyle}>Amount</th>
                <th style={thStyle}>{t('console.earnings.status')}</th>
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
        )}
      </div>
    </div>
  );
}