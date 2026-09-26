//! work_module_isolation — Dependency isolation for the semantic spec module.
//!
//! G1/G3 evidence: the semantic specification must not depend on composition
//! runtimes, harnesses, durable execution engines, or persistence engines. All
//! implementation coupling is confined to adapter/provider layers.

use std::fs;
use std::path::Path;

const FORBIDDEN_MARKERS: [&str; 8] = [
    "cordis",
    "deepseek",
    "harness_dsh",
    "dsh::",
    "temporal",
    "kafka",
    "rusqlite",
    "sled::",
];

#[test]
fn semantic_spec_module_has_no_runtime_or_persistence_coupling() {
    let work_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/work");
    let mut checked = 0;
    let mut violations = Vec::new();

    let entries = fs::read_dir(&work_dir).expect("src/work must exist");
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        checked += 1;
        let content = fs::read_to_string(&path).unwrap_or_default();
        for marker in FORBIDDEN_MARKERS {
            if content.contains(marker) {
                violations.push(format!(
                    "{} contains forbidden coupling marker '{marker}'",
                    path.display()
                ));
            }
        }
    }

    assert!(
        checked >= 10,
        "expected the full semantic module tree to be scanned, checked {checked}"
    );
    assert!(
        violations.is_empty(),
        "semantic spec module must stay provider-free:\n{}",
        violations.join("\n")
    );
}
