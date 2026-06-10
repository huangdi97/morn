use std::fs;
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
        let _ = (path, dest);
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
        let _ = name;
        Ok(())
    }

    pub fn close_app(name: &str) -> Result<(), String> {
        let _ = name;
        Ok(())
    }

    pub fn list_running_apps() -> Result<Vec<String>, String> {
        Ok(vec![])
    }
}

/// Desktop control
impl ComputerControl {
    pub fn screenshot() -> Result<Vec<u8>, String> {
        Ok(vec![])
    }

    pub fn type_text(text: &str) -> Result<(), String> {
        let _ = text;
        Ok(())
    }

    pub fn get_clipboard() -> Result<String, String> {
        Ok(String::new())
    }

    pub fn set_clipboard(text: &str) -> Result<(), String> {
        let _ = text;
        Ok(())
    }
}
