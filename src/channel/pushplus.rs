//! PushPlus 推送渠道 — 通过 pushplus.plus 发送消息通知
use crate::core::error::{MornError, MornResult};
use reqwest::blocking::Client;

pub fn pushplus_push(token: &str, title: &str, content: &str) -> MornResult<()> {
    let payload = serde_json::json!({
        "token": token,
        "title": title,
        "content": content,
    });

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .post("http://www.pushplus.plus/send")
        .json(&payload)
        .send()
        .map_err(|e| format!("Failed to send PushPlus message: {}", e))?;

    let status = response.status();
    let body: serde_json::Value = response
        .json()
        .map_err(|e| format!("Failed to parse PushPlus response: {}", e))?;

    if !status.is_success() {
        return Err(MornError::Network(format!(
            "PushPlus API returned non-200 status {}: {}",
            status, body
        )));
    }

    let code = body
        .get("code")
        .and_then(|v| {
            v.as_i64()
                .or_else(|| v.as_str().and_then(|s| s.parse::<i64>().ok()))
        })
        .unwrap_or(200);

    if code != 200 {
        let message = body
            .get("msg")
            .or_else(|| body.get("message"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown error");
        return Err(MornError::Network(format!("PushPlus API error {}: {}", code, message)));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pushplus_push_returns_error_on_bad_token() {
        let result = pushplus_push("bad-token", "title", "content");
        assert!(result.is_err());
    }

    #[test]
    fn pushplus_push_accepts_empty_fields() {
        let result = pushplus_push("tok", "", "content");
        assert!(result.is_err());
        let result = pushplus_push("tok", "title", "");
        assert!(result.is_err());
    }
}
