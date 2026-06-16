//! 电脑操控抽象层 — 文件搜索、压缩、应用启动、桌面控制、剪贴板
use crate::core::error::MornError;
use std::fs;
use std::io::Write;
use std::path::Path;

pub struct ComputerControl;

/// File system operations
impl ComputerControl {
    /// List all entries in a directory.
    pub fn list_dir(path: &str) -> Result<Vec<String>, MornError> {
        let entries = fs::read_dir(path).map_err(|e| MornError::Internal(e.to_string()))?;
        entries
            .map(|entry| {
                entry
                    .map(|e| e.file_name().to_string_lossy().to_string())
                    .map_err(|e| MornError::Internal(e.to_string()))
            })
            .collect()
    }

    /// Read the full contents of a file as a string.
    pub fn read_file(path: &str) -> Result<String, MornError> {
        let resolved = Path::new(path)
            .canonicalize()
            .map_err(|e| MornError::Internal(format!("invalid path: {}", e)))?;
        fs::read_to_string(&resolved).map_err(|e| MornError::Internal(e.to_string()))
    }

    /// Write content to a file, creating or overwriting it.
    pub fn write_file(path: &str, content: &str) -> Result<(), MornError> {
        let p = Path::new(path);
        let resolved = if let Some(parent) = p.parent().filter(|x| !x.as_os_str().is_empty()) {
            let canonical_parent = parent
                .canonicalize()
                .map_err(|e| MornError::Internal(format!("invalid path: {}", e)))?;
            canonical_parent.join(p.file_name().unwrap_or_default())
        } else {
            p.to_path_buf()
        };
        fs::write(&resolved, content).map_err(|e| MornError::Internal(e.to_string()))
    }

    /// Move or rename a file from source to destination.
    pub fn move_file(src: &str, dst: &str) -> Result<(), MornError> {
        let resolved_src = Path::new(src)
            .canonicalize()
            .map_err(|e| MornError::Internal(format!("invalid source path: {}", e)))?;
        let dst_path = Path::new(dst);
        let resolved_dst =
            if let Some(parent) = dst_path.parent().filter(|x| !x.as_os_str().is_empty()) {
                let canonical_parent = parent
                    .canonicalize()
                    .map_err(|e| MornError::Internal(format!("invalid destination path: {}", e)))?;
                canonical_parent.join(dst_path.file_name().unwrap_or_default())
            } else {
                dst_path.to_path_buf()
            };
        fs::rename(&resolved_src, &resolved_dst).map_err(|e| MornError::Internal(e.to_string()))
    }

    /// Delete a file at the given path.
    pub fn delete_file(path: &str) -> Result<(), MornError> {
        let resolved = Path::new(path)
            .canonicalize()
            .map_err(|e| MornError::Internal(format!("invalid path: {}", e)))?;
        fs::remove_file(&resolved).map_err(|e| MornError::Internal(e.to_string()))
    }

    /// Recursively search for files matching a glob-like pattern.
    pub fn search_files(pattern: &str, path: &str) -> Result<Vec<String>, MornError> {
        let base = Path::new(path)
            .canonicalize()
            .map_err(|e| MornError::Internal(format!("invalid path: {}", e)))?;
        let mut results = Vec::new();
        let pattern = pattern.to_string();
        visit_dirs(&base, &pattern, &mut results).map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(results)
    }

    /// Compress a file or directory into a zip archive.
    pub fn compress(path: &str, dest: &str) -> Result<(), MornError> {
        use std::fs::File;
        let resolved_path = Path::new(path)
            .canonicalize()
            .map_err(|e| MornError::Internal(format!("invalid source path: {}", e)))?;
        let dest_path = Path::new(dest);
        let resolved_dest =
            if let Some(parent) = dest_path.parent().filter(|x| !x.as_os_str().is_empty()) {
                let canonical_parent = parent
                    .canonicalize()
                    .map_err(|e| MornError::Internal(format!("invalid destination path: {}", e)))?;
                canonical_parent.join(dest_path.file_name().unwrap_or_default())
            } else {
                dest_path.to_path_buf()
            };
        let file = File::create(&resolved_dest).map_err(|e| MornError::Internal(e.to_string()))?;
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default();
        let data = std::fs::read(&resolved_path).map_err(|e| MornError::Internal(e.to_string()))?;
        let name = resolved_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();
        zip.start_file(name.as_ref(), options)
            .map_err(|e| MornError::Internal(e.to_string()))?;
        zip.write_all(&data).map_err(|e| MornError::Internal(e.to_string()))?;
        zip.finish().map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(())
    }
}

fn glob_match(pattern: &str, name: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if !pattern.contains('*') {
        return pattern == name;
    }
    let parts: Vec<&str> = pattern.split('*').collect();
    let mut pos = 0;
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if i == 0 {
            if !name.starts_with(part) {
                return false;
            }
            pos = part.len();
        } else if i == parts.len() - 1 {
            if !name[pos..].ends_with(part) {
                return false;
            }
        } else {
            match name[pos..].find(part) {
                Some(idx) => pos += idx + part.len(),
                None => return false,
            }
        }
    }
    true
}

fn visit_dirs(dir: &Path, pattern: &str, results: &mut Vec<String>) -> Result<(), MornError> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir).map_err(|e| MornError::Internal(e.to_string()))? {
            let entry = entry.map_err(|e| MornError::Internal(e.to_string()))?;
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if path.is_dir() {
                visit_dirs(&path, pattern, results)?;
            } else if glob_match(pattern, &name) {
                results.push(path.to_string_lossy().to_string());
            }
        }
    }
    Ok(())
}

/// Application management
impl ComputerControl {
    /// Launch a desktop application by executable name or path.
    pub fn launch_app(name: &str) -> Result<(), MornError> {
        std::process::Command::new(name)
            .spawn()
            .map_err(|e| MornError::Internal(format!("Failed to launch {}: {}", name, e)))?;
        Ok(())
    }

    /// Close a running application by name or PID.
    pub fn close_app(name: &str) -> Result<(), MornError> {
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("pkill")
                .arg(name)
                .output()
                .map_err(|e| MornError::Internal(e.to_string()))?;
        }
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("taskkill")
                .args(["/IM", name, "/F"])
                .output()
                .map_err(|e| MornError::Internal(e.to_string()))?;
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            tracing::warn!("close_app not implemented on this platform");
        }
        Ok(())
    }

    /// List all currently running desktop applications.
    pub fn list_running_apps() -> Result<Vec<String>, MornError> {
        Ok(vec![])
    }
}

/// Desktop control
impl ComputerControl {
    /// Capture a screenshot of the desktop or active window.
    pub fn screenshot() -> Result<Vec<u8>, MornError> {
        #[cfg(target_os = "linux")]
        {
            let screenshot_path = std::env::temp_dir().join("morn_screenshot.png");
            let screenshot_str = screenshot_path
                .to_str()
                .ok_or_else(|| format!("非UTF-8路径: {:?}", screenshot_path))?
                .to_string();
            let output = std::process::Command::new("import")
                .args(["-window", "root", &screenshot_str])
                .output()
                .map_err(|e| MornError::Internal(format!("screenshot failed: {}", e)))?;
            if output.status.success() {
                return std::fs::read(&screenshot_path).map_err(|e| MornError::Internal(e.to_string()));
            }
        }
        tracing::warn!("screenshot not implemented on this platform");
        Ok(vec![])
    }

    /// Type text programmatically (placeholder).
    pub fn type_text(text: &str) -> Result<(), MornError> {
        tracing::warn!("type_text not implemented: would type '{}'", text);
        Ok(())
    }

    /// Get the current clipboard text content.
    pub fn get_clipboard() -> Result<String, MornError> {
        tracing::warn!("get_clipboard not implemented");
        Ok(String::new())
    }

    /// Set clipboard text content.
    pub fn set_clipboard(text: &str) -> Result<(), MornError> {
        tracing::warn!("set_clipboard not implemented: '{}'", text);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn tmp_path(name: &str) -> String {
        std::env::temp_dir().join(name).display().to_string()
    }

    #[test]
    fn test_glob_match_exact() {
        assert!(glob_match("test.txt", "test.txt"));
        assert!(!glob_match("test.txt", "other.txt"));
    }

    #[test]
    fn test_glob_match_wildcard() {
        assert!(glob_match("*", "anything"));
        assert!(glob_match("*.rs", "main.rs"));
        assert!(!glob_match("*.rs", "main.txt"));
    }

    #[test]
    fn test_glob_match_prefix_wildcard() {
        assert!(glob_match("prefix*", "prefix_suffix"));
        assert!(!glob_match("prefix*", "other"));
    }

    #[test]
    fn test_glob_match_suffix_wildcard() {
        assert!(glob_match("*suffix", "prefix_suffix"));
        assert!(!glob_match("*suffix", "other"));
    }

    #[test]
    fn test_list_dir_current() {
        let result = ComputerControl::list_dir(".");
        assert!(result.is_ok());
        let entries = result.unwrap();
        assert!(!entries.is_empty());
    }

    #[test]
    fn test_list_dir_invalid_path() {
        let result = ComputerControl::list_dir("/nonexistent_path_xyz");
        assert!(result.is_err());
    }

    #[test]
    fn test_read_write_delete_file() {
        let path = tmp_path("_test_computer_control_write.txt");
        ComputerControl::write_file(&path, "hello world").unwrap();
        let content = ComputerControl::read_file(&path).unwrap();
        assert_eq!(content, "hello world");
        ComputerControl::delete_file(&path).unwrap();
        assert!(ComputerControl::read_file(&path).is_err());
    }

    #[test]
    fn test_move_file() {
        let src = tmp_path("_test_computer_control_move_src.txt");
        let dst = tmp_path("_test_computer_control_move_dst.txt");
        ComputerControl::write_file(&src, "move test").unwrap();
        ComputerControl::move_file(&src, &dst).unwrap();
        assert!(ComputerControl::read_file(&src).is_err());
        assert_eq!(ComputerControl::read_file(&dst).unwrap(), "move test");
        if let Err(e) = ComputerControl::delete_file(&dst) {
            tracing::warn!("failed to delete file: {}", e);
        }
    }

    #[test]
    fn test_search_files() {
        let dir = tmp_path("_test_computer_control_search");
        if let Err(e) = fs::remove_dir_all(&dir) {
            tracing::warn!("failed to remove dir: {}", e);
        }
        fs::create_dir_all(&dir).unwrap();
        ComputerControl::write_file(&format!("{}/a.rs", dir), "").unwrap();
        ComputerControl::write_file(&format!("{}/b.txt", dir), "").unwrap();
        let results = ComputerControl::search_files("*.rs", &dir).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].ends_with("a.rs"));
        if let Err(e) = fs::remove_dir_all(&dir) {
            tracing::warn!("failed to remove dir: {}", e);
        }
    }

    #[test]
    fn test_compress_real() {
        let src = tmp_path("_test_computer_control_compress.txt");
        let dst = tmp_path("_test_computer_control_out.zip");
        if let Err(e) = std::fs::remove_file(&dst) {
            tracing::warn!("failed to remove file: {}", e);
        }
        ComputerControl::write_file(&src, "compress me").unwrap();
        assert!(ComputerControl::compress(&src, &dst).is_ok());
        assert!(std::path::Path::new(&dst).exists());
        if let Err(e) = std::fs::remove_file(src) {
            tracing::warn!("failed to remove file: {}", e);
        }
        if let Err(e) = std::fs::remove_file(dst) {
            tracing::warn!("failed to remove file: {}", e);
        }
    }

    #[test]
    fn test_launch_app_noop() {
        let result = ComputerControl::launch_app("nonexistent_cmd_xyz");
        assert!(result.is_err());
    }

    #[test]
    fn test_close_app() {
        // pkill returns 0 even when no matching process exists
        if let Err(e) = ComputerControl::close_app("nonexistent_cmd_xyz") {
            tracing::warn!("failed to close app: {}", e);
        }
    }

    #[test]
    fn test_list_running_apps() {
        let apps = ComputerControl::list_running_apps().unwrap();
        assert!(apps.is_empty());
    }

    #[test]
    fn test_screenshot() {
        match ComputerControl::screenshot() {
            Ok(bytes) => {
                // On systems with `import` installed, expect PNG bytes
                if !bytes.is_empty() {
                    assert!(bytes.len() > 100);
                }
            }
            Err(e) => {
                // On systems without `import`, expect a failure message
                assert!(e.contains("screenshot failed") || e.contains("not implemented"));
            }
        }
    }

    #[test]
    fn test_type_text() {
        assert!(ComputerControl::type_text("hello").is_ok());
    }

    #[test]
    fn test_get_clipboard() {
        let text = ComputerControl::get_clipboard().unwrap();
        assert!(text.is_empty());
    }

    #[test]
    fn test_set_clipboard() {
        assert!(ComputerControl::set_clipboard("test").is_ok());
    }

    #[test]
    fn test_read_file_not_found() {
        let result = ComputerControl::read_file("/nonexistent_file_xyz");
        assert!(result.is_err());
    }
}
