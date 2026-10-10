//! Scrubbed environment for external harness subprocesses.
//!
//! Provider runtimes are an execution/credential boundary (ADR-009). They
//! must not inherit arbitrary Morn server secrets merely because they are
//! children of the application process.

use std::collections::BTreeSet;
use std::ffi::OsString;

/// Minimal process/runtime variables that are not provider credentials.
///
/// HOME/USERPROFILE and provider API keys are deliberately absent: Morn gives
/// DSH an explicit DSH_HOME, while additional credential/proxy variables must
/// be named explicitly by the deployment.
const BASE_ENVIRONMENT_NAMES: &[&str] = &[
    "PATH",
    "PATHEXT",
    "SYSTEMROOT",
    "SystemRoot",
    "WINDIR",
    "COMSPEC",
    "TEMP",
    "TMP",
    "TMPDIR",
    "LANG",
    "LC_ALL",
    "SSL_CERT_FILE",
    "SSL_CERT_DIR",
];

pub(crate) fn allowed_environment_names(extra: Option<&str>) -> BTreeSet<String> {
    let mut names: BTreeSet<String> = BASE_ENVIRONMENT_NAMES
        .iter()
        .map(|name| (*name).to_string())
        .collect();
    if let Some(extra) = extra {
        for raw in extra.split([',', ';']) {
            let name = raw.trim();
            if is_safe_environment_name(name) {
                names.insert(name.to_string());
            }
        }
    }
    names
}

pub(crate) fn scrubbed_environment(control_variable: &str) -> Vec<(OsString, OsString)> {
    let extra = std::env::var(control_variable).ok();
    allowed_environment_names(extra.as_deref())
        .into_iter()
        .filter_map(|name| std::env::var_os(&name).map(|value| (OsString::from(name), value)))
        .collect()
}

fn is_safe_environment_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 128
        && name
            .bytes()
            .all(|byte| byte == b'_' || byte.is_ascii_alphanumeric())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_only_explicit_environment_names() {
        let names =
            allowed_environment_names(Some("DEEPSEEK_API_KEY; HTTPS_PROXY, BAD-NAME, ,CUSTOM_KEY"));
        assert!(names.contains("PATH"));
        assert!(names.contains("DEEPSEEK_API_KEY"));
        assert!(names.contains("HTTPS_PROXY"));
        assert!(names.contains("CUSTOM_KEY"));
        assert!(!names.contains("BAD-NAME"));
        assert!(!names.contains("GITHUB_TOKEN"));
    }
}
