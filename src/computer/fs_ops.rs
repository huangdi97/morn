//! fs_ops — Provides filesystem operations for the computer control layer.
use crate::core::error::MornError;
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

    fn temp_path(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("morn_fs_ops_{}_{}", name, uuid::Uuid::new_v4()))
    }

    #[test]
    fn write_and_read_file_round_trip() {
        let path = temp_path("round_trip");
        let path_str = path.to_string_lossy().to_string();

        let write_result = write(&path_str, "hello");
        assert!(write_result.success);
        assert_eq!(
            write_result.security_level,
            SecurityLevel::L1Sandbox.as_str()
        );

        let read_result = read(&path_str);
        assert!(read_result.success);
        assert_eq!(read_result.data, "hello");

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn read_missing_file_reports_error() {
        let path = temp_path("missing");
        let result = read(&path.to_string_lossy());
        assert!(!result.success);
        assert_eq!(result.security_level, SecurityLevel::L1Sandbox.as_str());
    }

    #[test]
    fn move_file_changes_path() {
        let src = temp_path("move_src");
        let dst = temp_path("move_dst");
        std::fs::write(&src, "move").unwrap();

        let result = r#move(&src.to_string_lossy(), &dst.to_string_lossy());
        assert!(result.success);
        assert_eq!(result.security_level, SecurityLevel::L2Local.as_str());
        assert!(dst.exists());
        assert!(!src.exists());

        let _ = std::fs::remove_file(dst);
    }

    #[test]
    fn delete_file_requires_approval_on_success() {
        let path = temp_path("delete");
        std::fs::write(&path, "delete").unwrap();

        let result = delete(&path.to_string_lossy());
        assert!(result.success);
        assert_eq!(result.security_level, SecurityLevel::L3System.as_str());
        assert!(result.approval_required);
    }

    #[test]
    fn search_lists_matching_names() {
        let dir = temp_path("search_dir");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("alpha.txt"), "a").unwrap();
        std::fs::write(dir.join("beta.txt"), "b").unwrap();

        let result = search("alpha", &dir.to_string_lossy());
        assert!(result.success);
        assert!(result.data.contains("alpha.txt"));
        assert!(!result.data.contains("beta.txt"));

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn compress_existing_file_creates_archive() {
        let source = temp_path("compress_source");
        let dest = temp_path("compress_dest").with_extension("tar.gz");
        std::fs::write(&source, "archive").unwrap();

        let result = compress(&source.to_string_lossy(), &dest.to_string_lossy());
        assert_eq!(result.security_level, SecurityLevel::L1Sandbox.as_str());
        assert!(result.success);
        assert!(dest.exists());

        let _ = std::fs::remove_file(source);
        let _ = std::fs::remove_file(dest);
    }

    #[test]
    fn compress_missing_file_reports_failure() {
        let source = temp_path("missing_source");
        let dest = temp_path("missing_dest").with_extension("tar.gz");

        let result = compress(&source.to_string_lossy(), &dest.to_string_lossy());
        assert!(!result.success);
        assert_eq!(result.security_level, SecurityLevel::L1Sandbox.as_str());

        let _ = std::fs::remove_file(dest);
    }
}
