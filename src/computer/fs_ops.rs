use super::{ComputerOpResult, SecurityLevel};

pub fn read(path: &str) -> ComputerOpResult {
    match std::fs::read_to_string(path) {
        Ok(content) => ComputerOpResult {
            success: true,
            data: content,
            security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
            approval_required: false,
        },
        Err(e) => ComputerOpResult {
            success: false,
            data: format!("Error reading file: {}", e),
            security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
            approval_required: false,
        },
    }
}

pub fn write(path: &str, content: &str) -> ComputerOpResult {
    match std::fs::write(path, content) {
        Ok(_) => ComputerOpResult {
            success: true,
            data: format!("Written {} bytes to {}", content.len(), path),
            security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
            approval_required: false,
        },
        Err(e) => ComputerOpResult {
            success: false,
            data: format!("Error writing file: {}", e),
            security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
            approval_required: false,
        },
    }
}

pub fn r#move(src: &str, dst: &str) -> ComputerOpResult {
    match std::fs::rename(src, dst) {
        Ok(_) => ComputerOpResult {
            success: true,
            data: format!("Moved {} to {}", src, dst),
            security_level: SecurityLevel::L2Local.as_str().to_string(),
            approval_required: false,
        },
        Err(e) => ComputerOpResult {
            success: false,
            data: format!("Error moving file: {}", e),
            security_level: SecurityLevel::L2Local.as_str().to_string(),
            approval_required: false,
        },
    }
}

pub fn delete(path: &str) -> ComputerOpResult {
    match std::fs::remove_file(path) {
        Ok(_) => ComputerOpResult {
            success: true,
            data: format!("Deleted {}", path),
            security_level: SecurityLevel::L3System.as_str().to_string(),
            approval_required: true,
        },
        Err(e) => ComputerOpResult {
            success: false,
            data: format!("Error deleting file: {}", e),
            security_level: SecurityLevel::L3System.as_str().to_string(),
            approval_required: false,
        },
    }
}

pub fn search(pattern: &str, root: &str) -> ComputerOpResult {
    let mut results = Vec::new();
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.contains(pattern) {
                results.push(name);
            }
        }
    }
    ComputerOpResult {
        success: true,
        data: serde_json::to_string(&results).unwrap_or_default(),
        security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
        approval_required: false,
    }
}

pub fn compress(source: &str, dest: &str) -> ComputerOpResult {
    let result = std::process::Command::new("tar")
        .args(["-czf", dest, "-C"])
        .arg(
            std::path::Path::new(source)
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| ".".to_string()),
        )
        .arg(
            std::path::Path::new(source)
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| source.to_string()),
        )
        .output();

    match result {
        Ok(output) if output.status.success() => ComputerOpResult {
            success: true,
            data: format!("Compressed {} to {}", source, dest),
            security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
            approval_required: false,
        },
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            ComputerOpResult {
                success: false,
                data: format!("Compression failed: {}", stderr),
                security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
                approval_required: false,
            }
        }
        Err(e) => ComputerOpResult {
            success: false,
            data: format!("Compression error: {}", e),
            security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
            approval_required: false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_nonexistent_file() {
        let result = read("/tmp/nonexistent_file_xyz.txt");
        assert!(!result.success);
        assert!(result.data.contains("Error reading file"));
    }

    #[test]
    fn test_write_and_read_temp_file() {
        let path = "/tmp/morn_test_write.txt";
        let write_result = write(path, "hello world");
        assert!(write_result.success);
        assert!(write_result.data.contains("11 bytes"));

        let read_result = read(path);
        assert!(read_result.success);
        assert_eq!(read_result.data, "hello world");

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_move_nonexistent_file() {
        let result = r#move("/tmp/nonexistent_src", "/tmp/nonexistent_dst");
        assert!(!result.success);
    }

    #[test]
    fn test_delete_nonexistent_file() {
        let result = delete("/tmp/nonexistent_del_file");
        assert!(!result.success);
    }

    #[test]
    fn test_search_empty_directory() {
        let dir = "/tmp/morn_test_search";
        let _ = std::fs::create_dir_all(dir);
        let result = search("foo", dir);
        assert!(result.success);
        let files: Vec<String> = serde_json::from_str(&result.data).unwrap();
        assert!(files.is_empty());
        let _ = std::fs::remove_dir(dir);
    }

    #[test]
    fn test_search_finds_matching_file() {
        let dir = "/tmp/morn_test_search2";
        let _ = std::fs::create_dir_all(dir);
        let _ = std::fs::write(format!("{}/test_foo.txt", dir), "data");
        let _ = std::fs::write(format!("{}/bar.txt", dir), "data");

        let result = search("foo", dir);
        assert!(result.success);
        let files: Vec<String> = serde_json::from_str(&result.data).unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].contains("foo"));

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_compress_fails_without_tar() {
        let result = compress("/nonexistent/path", "/tmp/out.tar.gz");
        assert!(!result.success);
    }
}
