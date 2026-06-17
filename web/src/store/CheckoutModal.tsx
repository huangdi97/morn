import { useState } from "react";

interface CheckoutModalProps {
  name: string;
  icon: string;
  price: number;
  onConfirm: () => Promise<void>;
  onClose: () => void;
}

type PaymentMethod = "mock" | "stripe" | "alipay";
type CheckoutStatus = "idle" | "processing" | "success" | "error";

const PAYMENT_LABELS: Record<PaymentMethod, string> = {
  mock: "模拟支付 (Mock)",
  stripe: "Stripe",
  alipay: "支付宝",
};

export default function CheckoutModal({ name, icon, price, onConfirm, onClose }: CheckoutModalProps) {
  const [paymentMethod, setPaymentMethod] = useState<PaymentMethod>("mock");
  const [status, setStatus] = useState<CheckoutStatus>("idle");
  const [errorMsg, setErrorMsg] = useState("");

  const handleConfirm = async () => {
    setStatus("processing");
    setErrorMsg("");
    try {
      await onConfirm();
      setStatus("success");
    } catch (e: any) {
      setStatus("error");
      setErrorMsg(e?.toString() || "Purchase failed");
    }
  };

  return (
    <div style={{
      position: "fixed", top: 0, left: 0, right: 0, bottom: 0,
      background: "rgba(0,0,0,0.7)", zIndex: 1000,
      display: "flex", alignItems: "center", justifyContent: "center",
    }} onClick={onClose}>
      <div style={{
        background: "var(--bg-surface)", borderRadius: "var(--radius-xl)",
        border: "1px solid var(--border-default)", padding: "32px",
        maxWidth: "420px", width: "90%",
      }} onClick={e => e.stopPropagation()}>
        {status === "success" ? (
          <div style={{ textAlign: "center" }}>
            <div style={{ fontSize: "48px", marginBottom: "12px" }}>✓</div>
            <h3 style={{ color: "#3fb950", margin: "0 0 8px" }}>Purchase Successful</h3>
            <p style={{ color: "#8b949e", fontSize: "13px", margin: "0 0 20px" }}>
              {name} has been installed.
            </p>
            <button onClick={onClose}
              style={{
                padding: "8px 24px", background: "#1f6feb", color: "#fff",
                border: "none", borderRadius: "4px", cursor: "pointer", fontSize: "13px",
              }}>
              Close
            </button>
          </div>
        ) : (
          <>
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "20px" }}>
              <h3 style={{ color: "#e6edf3", margin: 0 }}>Checkout</h3>
              <button onClick={onClose}
                style={{
                  background: "none", border: "none", color: "#8b949e",
                  cursor: "pointer", fontSize: "18px", padding: "4px",
                }}>
                ✕
              </button>
            </div>

            <div style={{
              display: "flex", alignItems: "center", gap: "12px",
              padding: "12px", background: "#0d1117", borderRadius: "4px",
              marginBottom: "20px",
            }}>
              <span style={{ fontSize: "32px" }}>{icon}</span>
              <div>
                <div style={{ color: "#e6edf3", fontWeight: "bold", fontSize: "14px" }}>{name}</div>
                <div style={{ color: "#f85149", fontSize: "15px", fontWeight: "bold" }}>
                  ¥{price.toFixed(3)}
                </div>
              </div>
            </div>

            <div style={{ marginBottom: "20px" }}>
              <div style={{ color: "#8b949e", fontSize: "12px", marginBottom: "8px" }}>
                Payment Method
              </div>
              <div style={{ display: "flex", flexDirection: "column", gap: "6px" }}>
                {(Object.keys(PAYMENT_LABELS) as PaymentMethod[]).map(m => (
                  <label key={m} style={{
                    display: "flex", alignItems: "center", gap: "8px",
                    padding: "8px 12px", background: paymentMethod === m ? "#1c2541" : "#0d1117",
                    border: paymentMethod === m ? "1px solid #58a6ff" : "1px solid #30363d",
                    borderRadius: "4px", cursor: "pointer", color: "#e6edf3", fontSize: "13px",
                  }}>
                    <input type="radio" name="payment" value={m} checked={paymentMethod === m}
                      onChange={() => setPaymentMethod(m)} />
                    {PAYMENT_LABELS[m]}
                  </label>
                ))}
              </div>
            </div>

            {status === "error" && (
              <div style={{
                padding: "8px 12px", borderRadius: "4px", fontSize: "13px",
                background: "#3d1111", color: "#f85149", marginBottom: "12px",
              }}>
                {errorMsg}
              </div>
            )}

            <button onClick={handleConfirm} disabled={status === "processing"}
              style={{
                width: "100%", padding: "10px", fontSize: "14px",
                background: status === "processing" ? "#21262d" : "#1f6feb",
                color: "#fff", border: "none", borderRadius: "4px",
                cursor: status === "processing" ? "default" : "pointer",
              }}>
              {status === "processing" ? "Processing..." : `Pay ¥${price.toFixed(3)}`}
            </button>
          </>
        )}
      </div>
    </div>
  );
}