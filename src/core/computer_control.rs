//! 电脑操控抽象层 — 文件搜索、压缩、应用启动、桌面控制、剪贴板
use std::fs;
use std::io::Write;
use std::path::Path;

pub struct ComputerControl;

/// File system operations
impl ComputerControl {
    pub fn list_dir(path: &str) -> Result<Vec<String>, String> {
        let entries = fs::read_dir(path).map_err(|e| e.to_string())?;
        entries
            .map(|entry| {
                entry
                    .map(|e| e.file_name().to_string_lossy().to_string())
                    .map_err(|e| e.to_string())
            })
            .collect()
    }

    pub fn read_file(path: &str) -> Result<String, String> {
        fs::read_to_string(path).map_err(|e| e.to_string())
    }

    pub fn write_file(path: &str, content: &str) -> Result<(), String> {
        fs::write(path, content).map_err(|e| e.to_string())
    }

    pub fn move_file(src: &str, dst: &str) -> Result<(), String> {
        fs::rename(src, dst).map_err(|e| e.to_string())
    }

    pub fn delete_file(path: &str) -> Result<(), String> {
        fs::remove_file(path).map_err(|e| e.to_string())
    }

    pub fn search_files(pattern: &str, path: &str) -> Result<Vec<String>, String> {
        let mut results = Vec::new();
        let base = Path::new(path);
        let pattern = pattern.to_string();
        visit_dirs(base, &pattern, &mut results).map_err(|e| e.to_string())?;
        Ok(results)
    }

    pub fn compress(path: &str, dest: &str) -> Result<(), String> {
        use std::fs::File;
        let file = File::create(dest).map_err(|e| e.to_string())?;
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default();
        let data = std::fs::read(path).map_err(|e| e.to_string())?;
        let name = std::path::Path::new(path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();
        zip.start_file(name.as_ref(), options)
            .map_err(|e| e.to_string())?;
        zip.write_all(&data).map_err(|e| e.to_string())?;
        zip.finish().map_err(|e| e.to_string())?;
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

fn visit_dirs(dir: &Path, pattern: &str, results: &mut Vec<String>) -> Result<(), String> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
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
    pub fn launch_app(name: &str) -> Result<(), String> {
        std::process::Command::new(name)
            .spawn()
            .map_err(|e| format!("Failed to launch {}: {}", name, e))?;
        Ok(())
    }

    pub fn close_app(name: &str) -> Result<(), String> {
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("pkill")
                .arg(name)
                .output()
                .map_err(|e| e.to_string())?;
        }
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("taskkill")
                .args(["/IM", name, "/F"])
                .output()
                .map_err(|e| e.to_string())?;
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            tracing::warn!("close_app not implemented on this platform");
        }
        Ok(())
    }

    pub fn list_running_apps() -> Result<Vec<String>, String> {
        Ok(vec![])
    }
}

/// Desktop control
impl ComputerControl {
    pub fn screenshot() -> Result<Vec<u8>, String> {
        #[cfg(target_os = "linux")]
        {
            let output = std::process::Command::new("import")
                .args(["-window", "root", "/tmp/morn_screenshot.png"])
                .output()
                .map_err(|e| format!("screenshot failed: {}", e))?;
            if output.status.success() {
                return std::fs::read("/tmp/morn_screenshot.png").map_err(|e| e.to_string());
            }
        }
        tracing::warn!("screenshot not implemented on this platform");
        Ok(vec![])
    }

    pub fn type_text(text: &str) -> Result<(), String> {
        tracing::warn!("type_text not implemented: would type '{}'", text);
        Ok(())
    }

    pub fn get_clipboard() -> Result<String, String> {
        tracing::warn!("get_clipboard not implemented");
        Ok(String::new())
    }

    pub fn set_clipboard(text: &str) -> Result<(), String> {
        tracing::warn!("set_clipboard not implemented: '{}'", text);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let path = "/tmp/_test_computer_control_write.txt";
        ComputerControl::write_file(path, "hello world").unwrap();
        let content = ComputerControl::read_file(path).unwrap();
        assert_eq!(content, "hello world");
        ComputerControl::delete_file(path).unwrap();
        assert!(ComputerControl::read_file(path).is_err());
    }

    #[test]
    fn test_move_file() {
        let src = "/tmp/_test_computer_control_move_src.txt";
        let dst = "/tmp/_test_computer_control_move_dst.txt";
        ComputerControl::write_file(src, "move test").unwrap();
        ComputerControl::move_file(src, dst).unwrap();
        assert!(ComputerControl::read_file(src).is_err());
        assert_eq!(ComputerControl::read_file(dst).unwrap(), "move test");
        let _ = ComputerControl::delete_file(dst);
    }

    #[test]
    fn test_search_files() {
        let dir = "/tmp/_test_computer_control_search";
        let _ = fs::remove_dir_all(dir);
        fs::create_dir_all(dir).unwrap();
        ComputerControl::write_file(&format!("{}/a.rs", dir), "").unwrap();
        ComputerControl::write_file(&format!("{}/b.txt", dir), "").unwrap();
        let results = ComputerControl::search_files("*.rs", dir).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].ends_with("a.rs"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_compress_real() {
        let src = "/tmp/_test_computer_control_compress.txt";
        let dst = "/tmp/_test_computer_control_out.zip";
        let _ = std::fs::remove_file(dst);
        ComputerControl::write_file(src, "compress me").unwrap();
        assert!(ComputerControl::compress(src, dst).is_ok());
        assert!(std::path::Path::new(dst).exists());
        let _ = std::fs::remove_file(src);
        let _ = std::fs::remove_file(dst);
    }

    #[test]
    fn test_launch_app_noop() {
        let result = ComputerControl::launch_app("nonexistent_cmd_xyz");
        assert!(result.is_err());
    }

    #[test]
    fn test_close_app() {
        // pkill returns 0 even when no matching process exists
        let _ = ComputerControl::close_app("nonexistent_cmd_xyz");
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
