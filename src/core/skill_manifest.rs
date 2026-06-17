//! skill_manifest — Parses and represents skill manifest metadata.
use crate::core::error::MornError;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: Option<String>,
    pub tools: Vec<String>,
    pub dependencies: Vec<String>,
    pub tags: Vec<String>,
    pub source_file: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SkillLoader {
    scan_dirs: Vec<PathBuf>,
}

impl SkillLoader {
    pub fn new(scan_dirs: Vec<PathBuf>) -> Self {
        SkillLoader { scan_dirs }
    }

    pub fn discover(&self) -> Result<Vec<SkillManifest>, MornError> {
        let mut manifests = Vec::new();
        for dir in &self.scan_dirs {
            if !dir.exists() {
                continue;
            }
            let entries = fs::read_dir(dir)
                .map_err(|e| MornError::Internal(format!("Cannot read dir {:?}: {}", dir, e)))?;
            for entry in entries {
                let entry =
                    entry.map_err(|e| MornError::Internal(format!("Entry error: {}", e)))?;
                let path = entry.path();
                if path.is_dir() {
                    let skill_file = path.join("SKILL.md");
                    if skill_file.exists() {
                        match self.parse_file(&skill_file) {
                            Ok(mut manifest) => {
                                manifest.source_file =
                                    Some(skill_file.to_string_lossy().to_string());
                                manifests.push(manifest);
                            }
                            Err(e) => {
                                tracing::info!(
                                    "[SkillLoader] Warning: failed to parse {:?}: {}",
                                    skill_file,
                                    e
                                );
                            }
                        }
                    }
                }
            }
        }
        Ok(manifests)
    }

    pub fn parse_file(&self, path: &Path) -> Result<SkillManifest, MornError> {
        let content = fs::read_to_string(path)
            .map_err(|e| MornError::Internal(format!("Cannot read {:?}: {}", path, e)))?;

        let frontmatter = parse_frontmatter(&content)?;
        let map: HashMap<String, String> = frontmatter.into_iter().collect();

        let id = map
            .get("id")
            .cloned()
            .or_else(|| {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| "unknown".to_string());

        let name = map.get("name").cloned().unwrap_or_default();
        let version = map
            .get("version")
            .cloned()
            .unwrap_or_else(|| "0.1.0".to_string());
        let description = map.get("description").cloned().unwrap_or_default();
        let author = map.get("author").cloned();

        let tools_str = map.get("tools").cloned().unwrap_or_default();
        let tools: Vec<String> = tools_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let deps_str = map.get("dependencies").cloned().unwrap_or_default();
        let dependencies: Vec<String> = deps_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let tags_str = map.get("tags").cloned().unwrap_or_default();
        let tags: Vec<String> = tags_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(SkillManifest {
            id,
            name,
            version,
            description,
            author,
            tools,
            dependencies,
            tags,
            source_file: None,
        })
    }
}

/// Parses structured key-value pairs from a YAML-like frontmatter block.
fn parse_frontmatter(content: &str) -> Result<Vec<(String, String)>, MornError> {
    let content = content.trim();
    if !content.starts_with("---") {
        return Err(MornError::Internal(
            "Missing frontmatter delimiters (---)".to_string(),
        ));
    }

    let rest = &content[3..];
    let end = rest
        .find("\n---")
        .ok_or_else(|| "Unclosed frontmatter delimiter".to_string())?;

    let frontmatter_str = &rest[..end];
    let mut fields = Vec::new();

    for line in frontmatter_str.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(pos) = line.find(':') {
            let key = line[..pos].trim().to_lowercase();
            let value = line[pos + 1..].trim().to_string();
            fields.push((key, value));
        }
    }

    Ok(fields)
}

pub fn create_default_skill_md(
    path: &Path,
    id: &str,
    name: &str,
    description: &str,
) -> Result<(), MornError> {
    let content = format!(
        "---\nid: {}\nname: {}\ndescription: {}\nversion: 0.1.0\nauthor: morn\ntools: \ndependencies: \ntags: \n---\n\n# {}\n\n{}",
        id, name, description, name, description
    );
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| MornError::Internal(format!("Cannot create dir: {}", e)))?;
    }
    fs::write(path, &content)
        .map_err(|e| MornError::Internal(format!("Cannot write {:?}: {}", path, e)))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn test_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("morn-test-{}", uuid::Uuid::new_v4()));
        if let Err(e) = fs::create_dir_all(&dir) {
            tracing::warn!("failed to create skill dir: {}", e);
        }
        dir
    }

    #[test]
    fn test_parse_skill_manifest() {
        let content = r#"---
id: test-skill
name: Test Skill
description: A test skill manifest
version: 1.0.0
author: test-author
tools: web_search, read_file
dependencies: 
tags: test, sample
---

# Test Skill

This is a test skill.
"#;

        let dir = test_dir();
        let file_path = dir.join("SKILL.md");
        fs::write(&file_path, content).unwrap();

        let loader = SkillLoader::new(vec![dir.clone()]);
        let result = loader.parse_file(&file_path).unwrap();

        assert_eq!(result.id, "test-skill");
        assert_eq!(result.name, "Test Skill");
        assert_eq!(result.description, "A test skill manifest");
        assert_eq!(result.version, "1.0.0");
        assert_eq!(result.author, Some("test-author".to_string()));
        assert_eq!(result.tools, vec!["web_search", "read_file"]);
        assert_eq!(result.tags, vec!["test", "sample"]);

        if let Err(e) = fs::remove_dir_all(&dir) {
            tracing::warn!("failed to remove dir: {}", e);
        }
    }

    #[test]
    fn test_discover_skills() {
        let dir = test_dir();
        let skill_dir = dir.join("my-skill");
        fs::create_dir_all(&skill_dir).unwrap();

        let content = r#"---
id: my-skill
name: My Skill
description: A discovered skill
version: 0.1.0
tools: calc
tags: utility
---
"#;
        fs::write(skill_dir.join("SKILL.md"), content).unwrap();

        let loader = SkillLoader::new(vec![dir.clone()]);
        let manifests = loader.discover().unwrap();

        assert_eq!(manifests.len(), 1);
        assert_eq!(manifests[0].id, "my-skill");

        if let Err(e) = fs::remove_dir_all(&dir) {
            tracing::warn!("failed to remove dir: {}", e);
        }
    }

    #[test]
    fn test_invalid_frontmatter() {
        let content = "No frontmatter here";
        let dir = test_dir();
        let file_path = dir.join("SKILL.md");
        fs::write(&file_path, content).unwrap();

        let loader = SkillLoader::new(vec![dir.clone()]);
        let result = loader.parse_file(&file_path);
        assert!(result.is_err());

        if let Err(e) = fs::remove_dir_all(&dir) {
            tracing::warn!("failed to remove dir: {}", e);
        }
    }

    #[test]
    fn test_create_default_skill() {
        let dir = test_dir();
        let path = dir.join("skills").join("test-skill").join("SKILL.md");
        create_default_skill_md(&path, "test-skill", "Test Skill", "A test").unwrap();

        assert!(path.exists());
        let loader = SkillLoader::new(vec![dir.join("skills")]);
        let manifest = loader.parse_file(&path).unwrap();
        assert_eq!(manifest.id, "test-skill");

        if let Err(e) = fs::remove_dir_all(&dir) {
            tracing::warn!("failed to remove dir: {}", e);
        }
    }
}
