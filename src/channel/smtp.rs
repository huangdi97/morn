//! smtp — Sends and receives channel messages through email transport.
use crate::core::error::MornError;
use crate::channel::adapter::ChannelMessage;

pub struct SmtpChannel {
    smtp_host: String,
    smtp_port: u16,
    username: String,
    password: String,
    from: String,
}

impl SmtpChannel {
    pub fn new(
        smtp_host: &str,
        smtp_port: u16,
        username: &str,
        password: &str,
        from: &str,
    ) -> Self {
        SmtpChannel {
            smtp_host: smtp_host.to_string(),
            smtp_port,
            username: username.to_string(),
            password: password.to_string(),
            from: from.to_string(),
        }
    }

    pub fn from_env() -> Result<Self, MornError> {
        let host = std::env::var("SMTP_HOST").map_err(|_| "SMTP_HOST not set".to_string())?;
        let port = std::env::var("SMTP_PORT")
            .unwrap_or_else(|_| "587".to_string())
            .parse::<u16>()
            .map_err(|e| MornError::Internal(format!("Invalid SMTP_PORT: {}", e)))?;
        let username =
            std::env::var("SMTP_USERNAME").map_err(|_| "SMTP_USERNAME not set".to_string())?;
        let password =
            std::env::var("SMTP_PASSWORD").map_err(|_| "SMTP_PASSWORD not set".to_string())?;
        let from = std::env::var("SMTP_FROM").map_err(|_| "SMTP_FROM not set".to_string())?;
        Ok(Self::new(&host, port, &username, &password, &from))
    }

    pub fn send_report(&self, to: &str, subject: &str, body: &str) -> Result<(), MornError> {
        use lettre::message::header::ContentType;
        use lettre::transport::smtp::authentication::Credentials;
        use lettre::{Message, SmtpTransport, Transport};

        let email = Message::builder()
            .from(
                self.from
                    .parse()
                    .map_err(|e: lettre::address::AddressError| {
                        format!("Invalid from address: {}", e)
                    })?,
            )
            .to(to
                .parse()
                .map_err(|e: lettre::address::AddressError| format!("Invalid to address: {}", e))?)
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .body(body.to_string())
            .map_err(|e| MornError::Internal(format!("Failed to build email: {}", e)))?;

        let creds = Credentials::new(self.username.clone(), self.password.clone());

        let mailer = SmtpTransport::starttls_relay(&self.smtp_host)
            .map_err(|e| MornError::Internal(format!("SMTP relay config error: {}", e)))?
            .port(self.smtp_port)
            .credentials(creds)
            .build();

        mailer
            .send(&email)
            .map_err(|e| MornError::Internal(format!("Failed to send email: {}", e)))?;

        Ok(())
    }

    pub fn send(&self, msg: &ChannelMessage) -> Result<(), MornError> {
        let to = msg
            .metadata
            .get("to")
            .and_then(|v| v.as_str())
            .unwrap_or("user@example.com");
        let subject = msg
            .metadata
            .get("subject")
            .and_then(|v| v.as_str())
            .unwrap_or("Morn Message");
        self.send_report(to, subject, &msg.content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smtp_new() {
        let channel = SmtpChannel::new("smtp.example.com", 587, "user", "pass", "from@test.com");
        assert_eq!(channel.smtp_host, "smtp.example.com");
        assert_eq!(channel.smtp_port, 587);
    }

    #[test]
    fn test_smtp_send_fails_without_server() {
        let channel = SmtpChannel::new("invalid.local", 25, "user", "pass", "from@test.com");
        let result = channel.send_report("to@test.com", "Test", "Body");
        assert!(result.is_err());
    }

    #[test]
    fn test_smtp_send_uses_metadata_defaults_path() {
        let channel = SmtpChannel::new("invalid.local", 25, "user", "pass", "from@test.com");
        let msg = ChannelMessage {
            content: "Body".into(),
            source: "smtp".into(),
            timestamp: 0,
            metadata: serde_json::json!({"to": "to@test.com", "subject": "Subject"}),
        };
        assert!(channel.send(&msg).is_err());
    }
}
