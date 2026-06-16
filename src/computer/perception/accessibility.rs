//! Accessibility tree perception — platform-specific accessibility queries.
use crate::core::error::MornError;
use crate::computer::{ComputerOpResult, SecurityLevel};

pub fn accessibility_tree() -> ComputerOpResult {
    let result = try_linux_a11y()
        .or_else(try_macos_a11y)
        .or_else(try_windows_a11y);

    match result {
        Some(tree) => ComputerOpResult {
            success: true,
            data: tree,
            security_level: SecurityLevel::L2Local.as_str().to_string(),
            approval_required: false,
        },
        None => ComputerOpResult {
            success: false,
            data: "No accessibility system available; install AT-SPI2 (Linux), enable Accessibility permissions (macOS), or use a supported platform".into(),
            security_level: SecurityLevel::L2Local.as_str().to_string(),
            approval_required: false,
        },
    }
}

fn try_linux_a11y() -> Option<String> {
    if !cfg!(target_os = "linux") {
        return None;
    }

    let script = r#"
import sys
try:
    import gi
    gi.require_version('Atspi', '2.0')
    from gi.repository import Atspi
    desktop = Atspi.get_desktop(0)
    def dump(node, depth=0):
        if not node:
            return ""
        try:
            name = node.get_name() or ""
            role = node.get_role_name() or "unknown"
            line = "  " * depth + f"[{role}] {name}"
        except:
            line = "  " * depth + "[unknown]"
        lines = [line]
        for i in range(node.get_child_count()):
            try:
                child = node.get_child(i)
                if child:
                    lines.append(dump(child, depth + 1))
            except:
                pass
        return "\n".join(lines)
    print(dump(desktop))
except Exception as e:
    print(f"error: {e}", file=sys.stderr)
    sys.exit(1)
"#;

    let output = std::process::Command::new("python3")
        .arg("-c")
        .arg(script)
        .output()
        .ok()?;
    if output.status.success() {
        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !text.is_empty() {
            return Some(format!("root →\n{}", text));
        }
    }

    let dbus_output = std::process::Command::new("dbus-send")
        .args([
            "--session",
            "--dest=org.a11y.atspi.Registry",
            "--print-reply",
            "/org/a11y/atspi/accessible/root",
            "org.a11y.atspi.Accessible.GetAccessibleChildren",
        ])
        .output()
        .ok()?;
    if dbus_output.status.success() {
        let text = String::from_utf8_lossy(&dbus_output.stdout)
            .trim()
            .to_string();
        if !text.is_empty() {
            return Some(format!("root [from AT-SPI2]\n{}", text));
        }
    }

    None
}

fn try_macos_a11y() -> Option<String> {
    if !cfg!(target_os = "macos") {
        return None;
    }

    let script = r#"
tell application "System Events"
    set appList to every process
    set output to ""
    repeat with proc in appList
        set procName to name of proc
        set output to output & "[application] " & procName & return
        try
            set winList to every window of proc
            repeat with w in winList
                set winName to title of w
                set output to output & "  [window] " & winName & return
            end repeat
        end try
    end repeat
    return output
end tell
"#;

    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .ok()?;
    if output.status.success() {
        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !text.is_empty() {
            return Some(format!("root →\n{}", text));
        }
    }

    let front_output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(r#"tell application "System Events" to get name of first process whose frontmost is true"#)
        .output()
        .ok()?;
    if front_output.status.success() {
        let text = String::from_utf8_lossy(&front_output.stdout)
            .trim()
            .to_string();
        if !text.is_empty() {
            return Some(format!("root → [application] {}", text));
        }
    }

    None
}

fn try_windows_a11y() -> Option<String> {
    if !cfg!(target_os = "windows") {
        return None;
    }

    let script = r#"
Add-Type -AssemblyName UIAutomationClient
$root = [System.Windows.Automation.Automation]::RootElement
$tree = ""
function Dump($el, $depth) {
    $name = $el.Current.Name
    $ctrl = $el.Current.ControlType.ProgrammaticName
    $tree += "  " * $depth + "[$ctrl] $name`n"
    $children = $el.FindAll("Subtree", $true)
    foreach ($child in $children) { Dump $child ($depth+1) }
}
Dump $root 0
Write-Output $tree
"#;

    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", script])
        .output()
        .ok()?;
    if output.status.success() {
        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !text.is_empty() {
            return Some(format!("root →\n{}", text));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessibility_tree_returns_or_errors_gracefully() {
        let result = accessibility_tree();
        assert_eq!(result.security_level, SecurityLevel::L2Local.as_str());
        assert!(!result.data.is_empty());
        assert!(!result.data.contains("simulated"));
    }
}
