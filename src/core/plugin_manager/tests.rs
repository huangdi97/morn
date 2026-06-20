use super::*;
use std::path::PathBuf;

fn temp_plugin_dir() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let plugin_dir = dir.path().join("plugins");
    std::fs::create_dir_all(&plugin_dir).unwrap();
    (dir, plugin_dir)
}

fn write_manifest(dir: &std::path::Path, name: &str, plugin_type: &str) {
    let manifest = serde_json::json!({
        "name": name,
        "version": "1.0.0",
        "description": format!("Test {}", name),
        "author": "Morn Labs",
        "plugin_type": plugin_type,
        "entry": "main.js"
    });
    std::fs::create_dir_all(dir.join(name)).unwrap();
    std::fs::write(
        dir.join(name).join("manifest.json"),
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();
}

#[test]
fn test_scan_finds_manifests() {
    let (_tmp, plugin_dir) = temp_plugin_dir();
    write_manifest(&plugin_dir, "theme-alpha", "theme");
    write_manifest(&plugin_dir, "channel-beta", "channel");
    let mut mgr = PluginManager::new(plugin_dir);
    let discovered = mgr.scan().unwrap();
    assert_eq!(discovered.len(), 2);
    assert!(discovered.contains(&"theme-alpha".to_string()));
    assert!(discovered.contains(&"channel-beta".to_string()));
}

#[test]
fn test_load_valid_manifest() {
    let (_tmp, plugin_dir) = temp_plugin_dir();
    write_manifest(&plugin_dir, "test-plugin", "theme");
    let mut mgr = PluginManager::new(plugin_dir);
    mgr.scan().unwrap();
    mgr.load("test-plugin").unwrap();
    let p = mgr.get("test-plugin").unwrap();
    assert_eq!(p.status, PluginStatus::Loaded);
}

#[test]
fn test_activate_deactivate_cycle() {
    let (_tmp, plugin_dir) = temp_plugin_dir();
    write_manifest(&plugin_dir, "cycle-plugin", "channel");
    let mut mgr = PluginManager::new(plugin_dir);
    mgr.scan().unwrap();
    mgr.load("cycle-plugin").unwrap();
    mgr.activate("cycle-plugin").unwrap();
    assert_eq!(
        mgr.get("cycle-plugin").unwrap().status,
        PluginStatus::Active
    );
    mgr.deactivate("cycle-plugin").unwrap();
    assert_eq!(
        mgr.get("cycle-plugin").unwrap().status,
        PluginStatus::Loaded
    );
}

#[test]
fn test_load_nonexistent_plugin() {
    let (_tmp, plugin_dir) = temp_plugin_dir();
    let mut mgr = PluginManager::new(plugin_dir);
    let result = mgr.load("ghost");
    assert!(result.is_err());
}

#[test]
fn test_scan_empty_dir() {
    let (_tmp, plugin_dir) = temp_plugin_dir();
    let mut mgr = PluginManager::new(plugin_dir);
    let discovered = mgr.scan().unwrap();
    assert!(discovered.is_empty());
}

#[test]
fn test_scan_ignores_dirs_without_manifest() {
    let (_tmp, plugin_dir) = temp_plugin_dir();
    std::fs::create_dir_all(plugin_dir.join("no-manifest")).unwrap();
    let mut mgr = PluginManager::new(plugin_dir);
    let discovered = mgr.scan().unwrap();
    assert!(discovered.is_empty());
}

#[test]
fn test_activate_theme_loads_css() {
    let (_tmp, plugin_dir) = temp_plugin_dir();
    let plugin_name = "dark-theme";
    let plugin_path = plugin_dir.join(plugin_name);
    std::fs::create_dir_all(&plugin_path).unwrap();

    let manifest = serde_json::json!({
        "name": plugin_name,
        "version": "1.0.0",
        "description": "A dark theme",
        "author": "Morn Labs",
        "plugin_type": "theme",
        "entry": "main.js"
    });
    std::fs::write(
        plugin_path.join("manifest.json"),
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();

    let css_content = "body { background: #000; color: #fff; }";
    std::fs::write(plugin_path.join("theme.css"), css_content).unwrap();

    let mut mgr = PluginManager::new(plugin_dir);
    mgr.scan().unwrap();
    mgr.activate(plugin_name).unwrap();

    assert_eq!(mgr.get_theme_css(plugin_name), Some(css_content));
}

#[test]
fn test_get_entry_path_returns_correct_path() {
    let (_tmp, plugin_dir) = temp_plugin_dir();
    write_manifest(&plugin_dir, "entry-test", "tool");
    let plugin_dir_clone = plugin_dir.clone();
    let mut mgr = PluginManager::new(plugin_dir);
    mgr.scan().unwrap();

    let entry = mgr.get_entry_path("entry-test");
    assert!(entry.is_some());
    let expected = plugin_dir_clone.join("entry-test").join("main.js");
    assert_eq!(entry.unwrap(), expected);
}

#[test]
fn test_list_themes_returns_only_themes() {
    let (_tmp, plugin_dir) = temp_plugin_dir();
    write_manifest(&plugin_dir, "theme-one", "theme");
    write_manifest(&plugin_dir, "theme-two", "theme");
    write_manifest(&plugin_dir, "channel-one", "channel");

    let mut mgr = PluginManager::new(plugin_dir);
    mgr.scan().unwrap();

    let themes = mgr.list_themes();
    assert_eq!(themes.len(), 2);
    assert!(themes.iter().any(|p| p.manifest.name == "theme-one"));
    assert!(themes.iter().any(|p| p.manifest.name == "theme-two"));
    assert!(!themes.iter().any(|p| p.manifest.name == "channel-one"));
}