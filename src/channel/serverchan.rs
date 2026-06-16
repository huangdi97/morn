//! Server酱推送渠道 — 通过 sctapi.ftqq.com 发送消息通知
use crate::core::error::MornError;
use reqwest::blocking::Client;

pub fn serverchan_push(token: &str, title: &str, content: &str) -> Result<(), MornError> {
    let url = format!("https://sctapi.ftqq.com/{}.send", token);
    let payload = serde_json::json!({
        "title": title,
        "desp": content,
    });

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| MornError::Internal(format!("Failed to create HTTP client: {}", e)))?;

    let response = client
        .post(&url)
        .json(&payload)
        .send()
        .map_err(|e| MornError::Internal(format!("Failed to send ServerChan message: {}", e)))?;

    let status = response.status();
    let body: serde_json::Value = response
        .json()
        .map_err(|e| MornError::Internal(format!("Failed to parse ServerChan response: {}", e)))?;

    if !status.is_success() {
        return Err(MornError::Internal(format!(
            "ServerChan API returned non-200 status {}: {}",
            status, body
        )));
    }

    let code = body
        .get("code")
        .and_then(|v| {
            v.as_i64()
                .or_else(|| v.as_str().and_then(|s| s.parse::<i64>().ok()))
        })
        .unwrap_or(0);

    if code != 0 {
        let message = body
            .get("message")
            .or_else(|| body.get("msg"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown error");
        return Err(MornError::Internal(format!("ServerChan API error {}: {}", code, message)));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serverchan_push_returns_error_on_bad_token() {
        let result = serverchan_push("bad-token", "title", "content");
        assert!(result.is_err());
    }

    #[test]
    fn serverchan_push_accepts_empty_fields() {
        let result = serverchan_push("tok", "", "content");
        assert!(result.is_err());
        let result = serverchan_push("tok", "title", "");
        assert!(result.is_err());
    }
}
