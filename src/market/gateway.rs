//! gateway — PaymentGateway trait and payment-related data types.
use crate::core::error::MornError;
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SubscriptionPlan {
    Free,
    Starter {
        amount: u64,
        currency: String,
    },
    Pro {
        amount: u64,
        currency: String,
        features: Vec<String>,
    },
    Enterprise {
        amount: u64,
        currency: String,
        seats: u32,
        contact_sales: bool,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PaymentUrl {
    pub url: String,
    pub payment_id: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PaymentStatus {
    Pending,
    Completed,
    Failed,
    Refunded,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PaymentError {
    Network(String),
    InvalidAmount,
    CurrencyMismatch,
    GatewayNotConfigured,
}

impl std::fmt::Display for PaymentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaymentError::Network(msg) => write!(f, "network error: {}", msg),
            PaymentError::InvalidAmount => write!(f, "invalid amount"),
            PaymentError::CurrencyMismatch => write!(f, "currency mismatch"),
            PaymentError::GatewayNotConfigured => write!(f, "gateway not configured"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subscription_plan_free_variant() {
        let plan = SubscriptionPlan::Free;
        assert!(matches!(plan, SubscriptionPlan::Free));
    }

    #[test]
    fn subscription_plan_starter_fields() {
        let plan = SubscriptionPlan::Starter {
            amount: 999,
            currency: "usd".into(),
        };
        match plan {
            SubscriptionPlan::Starter { amount, currency } => {
                assert_eq!(amount, 999);
                assert_eq!(currency, "usd");
            }
            _ => panic!("expected Starter"),
        }
    }

    #[test]
    fn payment_error_display() {
        assert_eq!(
            PaymentError::Network("timeout".into()).to_string(),
            "network error: timeout"
        );
        assert_eq!(PaymentError::InvalidAmount.to_string(), "invalid amount");
        assert_eq!(
            PaymentError::CurrencyMismatch.to_string(),
            "currency mismatch"
        );
        assert_eq!(
            PaymentError::GatewayNotConfigured.to_string(),
            "gateway not configured"
        );
    }
}

pub trait PaymentGateway: Send + Sync {
    fn create_payment(
        &self,
        amount: u64,
        currency: &str,
        description: &str,
    ) -> Result<PaymentUrl, PaymentError>;

    fn verify_payment(&self, payment_id: &str) -> Result<PaymentStatus, PaymentError>;

    fn process_refund(&self, payment_id: &str, amount: u64) -> Result<(), PaymentError>;
}
