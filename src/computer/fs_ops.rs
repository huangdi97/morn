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
