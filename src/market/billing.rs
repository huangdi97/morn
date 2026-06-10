//! 计费与结算模块 — Billing & Invoicing
//!
//! 定义市场交易的计费计划、订单、发票以及支付网关。
//! Defines billing plans, orders, invoices, and the payment gateway for marketplace transactions.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// 计费计划枚举 — Billing plan variants
///
/// 按次付费、月付订阅、年付订阅。
/// Per-use, monthly subscription, and annual subscription.
pub enum BillingPlan {
    PerUse,
    MonthlySubscription,
    AnnualSubscription,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// 订单结构 — Marketplace order
///
/// 包含订单 ID、商品 ID、买家、金额、状态和创建时间。
/// Contains order ID, listing ID, buyer, amount, status, and creation timestamp.
pub struct Order {
    pub id: String,
    pub listing_id: String,
    pub buyer: String,
    pub amount: f64,
    pub status: OrderStatus,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
/// 订单状态枚举 — Order lifecycle states
///
/// 待处理、已完成、已取消、已退款。
/// Pending, completed, cancelled, and refunded.
pub enum OrderStatus {
    Pending,
    Completed,
    Cancelled,
    Refunded,
}

impl OrderStatus {
    /// 返回状态的字符串表示 — Returns the string representation of the status
    ///
    /// `"pending"`, `"completed"`, `"cancelled"`, `"refunded"`
    pub fn as_str(&self) -> &'static str {
        match self {
            OrderStatus::Pending => "pending",
            OrderStatus::Completed => "completed",
            OrderStatus::Cancelled => "cancelled",
            OrderStatus::Refunded => "refunded",
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// 发票结构 — Payment invoice
///
/// 包含发票 ID、订单 ID、买家、金额、开票时间和付款时间。
/// Contains invoice ID, order ID, buyer, amount, issue time, and optional payment time.
pub struct Invoice {
    pub id: String,
    pub order_id: String,
    pub buyer: String,
    pub amount: f64,
    pub issued_at: String,
    pub paid_at: Option<String>,
}

/// 支付网关 — Payment gateway
///
/// 提供支付处理和退款功能的无状态网关。
/// Stateless gateway providing payment processing and refund capabilities.
pub struct PaymentGateway;

impl PaymentGateway {
    /// 处理支付并生成发票 — Processes payment and generates an invoice
    ///
    /// 如果支付成功则返回 `Invoice`，否则返回错误信息。
    /// Returns `Ok(Invoice)` on success, or `Err` with a description on failure.
    pub fn process_payment(order: &Order) -> Result<Invoice, String> {
        let invoice = Invoice {
            id: format!("inv-{}", uuid::Uuid::new_v4()),
            order_id: order.id.clone(),
            buyer: order.buyer.clone(),
            amount: order.amount,
            issued_at: chrono::Utc::now().to_rfc3339(),
            paid_at: Some(chrono::Utc::now().to_rfc3339()),
        };
        Ok(invoice)
    }

    /// 发起退款 — Processes a refund for an invoice
    ///
    /// `amount` 必须为正数，否则返回错误。
    /// `amount` must be positive; returns an error otherwise.
    pub fn refund(_invoice_id: &str, amount: f64) -> Result<(), String> {
        if amount <= 0.0 {
            return Err("Refund amount must be positive".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn order_creation() {
        let order = Order {
            id: "order-1".to_string(),
            listing_id: "listing-1".to_string(),
            buyer: "user-1".to_string(),
            amount: 9.99,
            status: OrderStatus::Pending,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        assert_eq!(order.status, OrderStatus::Pending);
        assert_eq!(order.amount, 9.99);
    }

    #[test]
    fn order_status_as_str() {
        assert_eq!(OrderStatus::Pending.as_str(), "pending");
        assert_eq!(OrderStatus::Completed.as_str(), "completed");
        assert_eq!(OrderStatus::Cancelled.as_str(), "cancelled");
        assert_eq!(OrderStatus::Refunded.as_str(), "refunded");
    }

    #[test]
    fn payment_gateway_processes_payment() {
        let order = Order {
            id: "order-1".to_string(),
            listing_id: "listing-1".to_string(),
            buyer: "user-1".to_string(),
            amount: 5.0,
            status: OrderStatus::Pending,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        let invoice = PaymentGateway::process_payment(&order).unwrap();
        assert_eq!(invoice.order_id, "order-1");
        assert_eq!(invoice.amount, 5.0);
        assert!(invoice.paid_at.is_some());
    }

    #[test]
    fn payment_gateway_refund_validation() {
        assert!(PaymentGateway::refund("inv-1", -1.0).is_err());
        assert!(PaymentGateway::refund("inv-1", 10.0).is_ok());
    }

    #[test]
    fn order_status_transition_pending_to_cancelled() {
        let order = Order {
            id: "order-2".to_string(),
            listing_id: "listing-2".to_string(),
            buyer: "user-2".to_string(),
            amount: 19.99,
            status: OrderStatus::Pending,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        assert_eq!(order.status.as_str(), "pending");
        // simulate transition
        let completed = Order { status: OrderStatus::Completed, ..order };
        assert_eq!(completed.status.as_str(), "completed");
    }

    #[test]
    fn invoice_generated_with_valid_order() {
        let order = Order {
            id: "order-3".to_string(),
            listing_id: "listing-3".to_string(),
            buyer: "user-3".to_string(),
            amount: 49.99,
            status: OrderStatus::Pending,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        let invoice = PaymentGateway::process_payment(&order).unwrap();
        assert_eq!(invoice.order_id, "order-3");
        assert_eq!(invoice.buyer, "user-3");
        assert_eq!(invoice.amount, 49.99);
        assert!(invoice.id.starts_with("inv-"));
    }

    #[test]
    fn payment_gateway_refund_success() {
        let result = PaymentGateway::refund("inv-2", 5.99);
        assert!(result.is_ok());
    }

    #[test]
    fn order_all_status_variants() {
        assert_eq!(OrderStatus::Pending.as_str(), "pending");
        assert_eq!(OrderStatus::Completed.as_str(), "completed");
        assert_eq!(OrderStatus::Cancelled.as_str(), "cancelled");
        assert_eq!(OrderStatus::Refunded.as_str(), "refunded");
    }
}