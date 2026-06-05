use crate::channel::adapter::ChannelMessage;

pub struct SmtpChannel {
    smtp_server: String,
    username: String,
    password: String,
}

impl SmtpChannel {
    pub fn new(smtp_server: &str, username: &str, password: &str) -> Self {
        SmtpChannel {
            smtp_server: smtp_server.to_string(),
            username: username.to_string(),
            password: password.to_string(),
        }
    }

    pub fn send_report(&self, to: &str, subject: &str, body: &str) -> Result<(), String> {
        Ok(())
    }

    pub fn send(&self, msg: &ChannelMessage) -> Result<(), String> {
        self.send_report("user@example.com", "Morn Message", &msg.content)
    }
}
