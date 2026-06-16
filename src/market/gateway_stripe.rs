//! gateway_stripe — Stripe PaymentGateway integration skeleton.
use crate::core::error::MornError;
use super::gateway::{PaymentError, PaymentGateway, PaymentStatus, PaymentUrl};

#[derive(Debug)]
pub struct StripePaymentGateway {
    _secret_key: String,
}

impl StripePaymentGateway {
    pub fn from_env() -> Result<Self, PaymentError> {
        let _secret_key =
            std::env::var("STRIPE_SECRET_KEY").map_err(|_| PaymentError::GatewayNotConfigured)?;
        Ok(Self { _secret_key })
    }
}

#[allow(unused_variables)]
impl PaymentGateway for StripePaymentGateway {
    fn create_payment(
        &self,
        amount: u64,
        currency: &str,
        description: &str,
    ) -> Result<PaymentUrl, PaymentError> {
        Err(PaymentError::GatewayNotConfigured)
    }

    fn verify_payment(&self, payment_id: &str) -> Result<PaymentStatus, PaymentError> {
        Err(PaymentError::GatewayNotConfigured)
    }

    fn process_refund(&self, payment_id: &str, amount: u64) -> Result<(), PaymentError> {
        Err(PaymentError::GatewayNotConfigured)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_env_no_var_returns_gateway_not_configured() {
        let original = std::env::var("STRIPE_SECRET_KEY").ok();
        std::env::remove_var("STRIPE_SECRET_KEY");
        let result = StripePaymentGateway::from_env();
        if let Some(key) = original {
            std::env::set_var("STRIPE_SECRET_KEY", key);
        }
        assert_eq!(result.unwrap_err(), PaymentError::GatewayNotConfigured);
    }

    #[test]
    fn constructor_does_not_panic() {
        let gw = StripePaymentGateway {
            _secret_key: "sk_test".into(),
        };
        assert_eq!(gw._secret_key, "sk_test");
    }
}
