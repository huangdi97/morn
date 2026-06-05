use super::{ComputerOpResult, SecurityLevel};

fn open_url(url: &str) -> Result<(), String> {
    if cfg!(target_os = "windows") {
        std::process::Command::new("cmd")
            .args(["/c", "start", url])
            .output()
            .map_err(|e| format!("Failed to open URL: {}", e))?;
    } else if cfg!(target_os = "linux") {
        std::process::Command::new("xdg-open")
            .arg(url)
            .output()
            .map_err(|e| format!("Failed to open URL: {}", e))?;
    } else if cfg!(target_os = "macos") {
        std::process::Command::new("open")
            .arg(url)
            .output()
            .map_err(|e| format!("Failed to open URL: {}", e))?;
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
            data: e,
            security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
            approval_required: false,
        },
    }
}

pub fn form_fill(selector: &str, value: &str) -> ComputerOpResult {
    ComputerOpResult {
        success: true,
        data: format!("[simulated] filled '{}' with '{}'", selector, value),
        security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
        approval_required: false,
    }
}

pub fn content_extract(url: &str) -> ComputerOpResult {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e));

    match client {
        Ok(client) => match client.get(url).send() {
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
        },
        Err(e) => ComputerOpResult {
            success: false,
            data: e,
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
            if i + 4 < chars.len() {
                if chars[i] == '&' {
                    if chars[i + 1] == 'a'
                        && chars[i + 2] == 'm'
                        && chars[i + 3] == 'p'
                        && chars[i + 4] == ';'
                    {
                        result.push('&');
                        i += 5;
                        continue;
                    }
                }
            }
            if i + 3 < chars.len() {
                if chars[i] == '&' {
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
