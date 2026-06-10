//! gateway_mock — Mock PaymentGateway for testing and development.
use super::gateway::{PaymentError, PaymentGateway, PaymentStatus, PaymentUrl};

pub struct MockPaymentGateway;

#[allow(unused_variables)]
impl PaymentGateway for MockPaymentGateway {
    fn create_payment(
        &self,
        amount: u64,
        currency: &str,
        description: &str,
    ) -> Result<PaymentUrl, PaymentError> {
        Ok(PaymentUrl {
            url: format!("https://mock.pay/{}/{}", currency, amount),
            payment_id: format!("mock_pay_{}", uuid::Uuid::new_v4()),
        })
    }

    fn verify_payment(&self, payment_id: &str) -> Result<PaymentStatus, PaymentError> {
        Ok(PaymentStatus::Completed)
    }

    fn process_refund(&self, payment_id: &str, amount: u64) -> Result<(), PaymentError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_payment_returns_payment_url() {
        let gw = MockPaymentGateway;
        let result = gw.create_payment(100, "usd", "test").unwrap();
        assert!(result.url.starts_with("https://mock.pay/usd/100"));
        assert!(result.payment_id.starts_with("mock_pay_"));
    }

    #[test]
    fn verify_payment_returns_completed() {
        let gw = MockPaymentGateway;
        let status = gw.verify_payment("any_id").unwrap();
        assert_eq!(status, PaymentStatus::Completed);
    }

    #[test]
    fn process_refund_ok() {
        let gw = MockPaymentGateway;
        assert!(gw.process_refund("any_id", 50).is_ok());
    }
}
