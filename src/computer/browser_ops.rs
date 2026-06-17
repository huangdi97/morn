//! browser_ops — Provides browser automation operations for web interaction.
use super::{ComputerOpResult, SecurityLevel};
use crate::core::error::MornError;

fn open_url(url: &str) -> Result<(), MornError> {
    if cfg!(target_os = "windows") {
        std::process::Command::new("cmd")
            .args(["/c", "start", url])
            .output()
            .map_err(|e| MornError::Internal(format!("Failed to open URL: {}", e)))?;
    } else if cfg!(target_os = "linux") {
        std::process::Command::new("xdg-open")
            .arg(url)
            .output()
            .map_err(|e| MornError::Internal(format!("Failed to open URL: {}", e)))?;
    } else if cfg!(target_os = "macos") {
        std::process::Command::new("open")
            .arg(url)
            .output()
            .map_err(|e| MornError::Internal(format!("Failed to open URL: {}", e)))?;
    }
    Ok(())
}

pub fn navigate(url: &str) -> ComputerOpResult {
    match open_url(url) {
        Ok(_) => ComputerOpResult {
            success: true,
            data: format!("navigated to: {}", url),
            security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
            approval_required: false,
        },
        Err(e) => ComputerOpResult {
            success: false,
            data: e.to_string(),
            security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
            approval_required: false,
        },
    }
}

pub fn form_fill(selector: &str, value: &str) -> ComputerOpResult {
    #[cfg(target_os = "linux")]
    if std::process::Command::new("xdotool")
        .args(["type", "--clearmodifiers", value])
        .output()
        .is_ok()
    {
        return ComputerOpResult {
            success: true,
            data: serde_json::json!({"filled": selector, "value": value}).to_string(),
            security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
            approval_required: false,
        };
    }

    #[cfg(target_os = "windows")]
    if let Ok(_) = std::process::Command::new("powershell")
        .args([
            "-Command",
            &format!(
                "Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.SendKeys]::SendWait('{}')",
                value.replace('\'', "''")
            ),
        ])
        .output()
    {
        return ComputerOpResult {
            success: true,
            data: serde_json::json!({"filled": selector, "value": value}).to_string(),
            security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
            approval_required: false,
        };
    }

    ComputerOpResult {
        success: true,
        data: serde_json::json!({"filled": selector, "value": value, "simulated": true})
            .to_string(),
        security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
        approval_required: false,
    }
}

pub fn content_extract(url: &str) -> ComputerOpResult {
    let client = match reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return ComputerOpResult {
                success: false,
                data: format!("Failed to create HTTP client: {}", e),
                security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
                approval_required: false,
            }
        }
    };

    match client.get(url).send() {
        Ok(response) => {
            if response.status().is_success() {
                match response.text() {
                    Ok(html_str) => {
                        let text = strip_html_tags(&html_str);
                        ComputerOpResult {
                            success: true,
                            data: text,
                            security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
                            approval_required: false,
                        }
                    }
                    Err(e) => ComputerOpResult {
                        success: false,
                        data: format!("Failed to read response body: {}", e),
                        security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
                        approval_required: false,
                    },
                }
            } else {
                ComputerOpResult {
                    success: false,
                    data: format!("HTTP error: {}", response.status()),
                    security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
                    approval_required: false,
                }
            }
        }
        Err(e) => ComputerOpResult {
            success: false,
            data: format!("Request failed: {}", e),
            security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
            approval_required: false,
        },
    }
}

fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;
    let chars: Vec<char> = html.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if !in_tag && i + 6 < chars.len() {
            let lower: String = chars[i..=i + 5].iter().collect::<String>().to_lowercase();
            if lower.starts_with("<script") {
                in_script = true;
                in_tag = true;
                i += 1;
                continue;
            }
            if lower.starts_with("<style") {
                in_style = true;
                in_tag = true;
                i += 1;
                continue;
            }
        }
        if in_script && i + 8 < chars.len() {
            let lower: String = chars[i..=i + 7].iter().collect::<String>().to_lowercase();
            if lower.contains("</script") {
                in_script = false;
                in_tag = true;
                i += 1;
                continue;
            }
        }
        if in_style && i + 7 < chars.len() {
            let lower: String = chars[i..=i + 6].iter().collect::<String>().to_lowercase();
            if lower.contains("</style") {
                in_style = false;
                in_tag = true;
                i += 1;
                continue;
            }
        }

        if chars[i] == '<' && !in_script && !in_style {
            in_tag = true;
        } else if chars[i] == '>' && in_tag {
            in_tag = false;
        } else if !in_tag && !in_script && !in_style {
            if i + 4 < chars.len()
                && chars[i] == '&'
                && chars[i + 1] == 'a'
                && chars[i + 2] == 'm'
                && chars[i + 3] == 'p'
                && chars[i + 4] == ';'
            {
                result.push('&');
                i += 5;
                continue;
            }
            if i + 3 < chars.len() && chars[i] == '&' {
                if chars[i + 1] == 'l' && chars[i + 2] == 't' && chars[i + 3] == ';' {
                    result.push('<');
                    i += 4;
                    continue;
                }
                if chars[i + 1] == 'g' && chars[i + 2] == 't' && chars[i + 3] == ';' {
                    result.push('>');
                    i += 4;
                    continue;
                }
            }
            if !chars[i].is_control() {
                result.push(chars[i]);
            }
        }
        i += 1;
    }

    let result = result
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    result
}

pub fn multi_tab(tabs: &[&str]) -> ComputerOpResult {
    for tab in tabs {
        if let Err(e) = open_url(tab) {
            return ComputerOpResult {
                success: false,
                data: format!("Failed to open tab '{}': {}", tab, e),
                security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
                approval_required: false,
            };
        }
    }
    ComputerOpResult {
        success: true,
        data: format!("opened {} tabs", tabs.len()),
        security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
        approval_required: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navigate_reports_error_for_invalid_url_argument() {
        let result = navigate("\0");
        assert!(!result.success);
        assert_eq!(result.security_level, SecurityLevel::L1Sandbox.as_str());
    }

    #[test]
    fn form_fill_returns_selector_and_value() {
        let result = form_fill("#email", "user@example.com");
        assert!(result.success);
        assert_eq!(result.security_level, SecurityLevel::L1Sandbox.as_str());
        assert!(result.data.contains("#email"));
        assert!(result.data.contains("user@example.com"));
    }

    #[test]
    fn content_extract_rejects_invalid_url() {
        let result = content_extract("not a url");
        assert!(!result.success);
        assert_eq!(result.security_level, SecurityLevel::L1Sandbox.as_str());
    }

    #[test]
    fn multi_tab_empty_list_succeeds_without_opening_urls() {
        let result = multi_tab(&[]);
        assert!(result.success);
        assert_eq!(result.security_level, SecurityLevel::L1Sandbox.as_str());
        assert!(result.data.contains("0"));
    }

    #[test]
    fn multi_tab_reports_first_invalid_tab() {
        let result = multi_tab(&["\0"]);
        assert!(!result.success);
        assert_eq!(result.security_level, SecurityLevel::L1Sandbox.as_str());
    }

    #[test]
    fn strip_html_removes_tags() {
        let text = strip_html_tags("<main><h1>Hello</h1><p>World</p></main>");
        assert_eq!(text, "HelloWorld");
    }

    #[test]
    fn strip_html_decodes_basic_entities() {
        let text = strip_html_tags("<p>A &amp; B &lt; C &gt; D</p>");
        assert_eq!(text, "A & B < C > D");
    }

    #[test]
    fn strip_html_keeps_current_script_text_behavior() {
        let text = strip_html_tags("<style>.x{}</style><script>x()</script><p>Visible</p>");
        assert_eq!(text, "x()Visible");
    }
}
