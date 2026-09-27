//! work_module_isolation — Dependency isolation for the semantic spec module.
//!
//! G1/G3 evidence: the semantic specification must not depend on composition
//! runtimes, harnesses, durable execution engines, or persistence engines. All
//! implementation coupling is confined to adapter/provider layers.

use std::fs;
use std::path::Path;

#[test]
fn semantic_spec_module_has_no_runtime_or_persistence_coupling() {
    // G3: the semantic spec AND the control plane must stay free of
    // composition-runtime coupling; only src/runtime adapters may import it.
    let manifest = env!("CARGO_MANIFEST_DIR");
    let work_dir = Path::new(manifest).join("src/work");
    let control_dir = Path::new(manifest).join("src/control");
    // Control-plane files legitimately mention "Cordis" in doc comments
    // describing the boundary; coupling detection is on the import path.
    let markers: [&str; 8] = [
        "cordis::",
        "use cordis",
        "deepseek",
        "harness_dsh",
        "dsh::",
        "temporal::",
        "kafka::",
        "sled::",
    ];
    let mut checked = 0;
    let mut violations = Vec::new();

    for dir in [work_dir, control_dir] {
        let entries = fs::read_dir(&dir).expect("module dir must exist");
        for entry in entries.flatten() {
            let path = entry.path();
            // Controllers live in a subdirectory; walk one level deep.
            if path.is_dir() {
                let sub_entries = fs::read_dir(&path).expect("controllers dir");
                for sub in sub_entries.flatten() {
                    let sub_path = sub.path();
                    if sub_path.extension().and_then(|e| e.to_str()) != Some("rs") {
                        continue;
                    }
                    checked += 1;
                    let content = fs::read_to_string(&sub_path).unwrap_or_default();
                    for marker in markers {
                        if content.contains(marker) {
                            violations.push(format!(
                                "{} contains forbidden coupling marker '{marker}'",
                                sub_path.display()
                            ));
                        }
                    }
                }
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            checked += 1;
            let content = fs::read_to_string(&path).unwrap_or_default();
            for marker in markers {
                if content.contains(marker) {
                    violations.push(format!(
                        "{} contains forbidden coupling marker '{marker}'",
                        path.display()
                    ));
                }
            }
        }
    }

    assert!(
        checked >= 20,
        "expected the full semantic + control-plane tree to be scanned, checked {checked}"
    );
    assert!(
        violations.is_empty(),
        "semantic/control-plane modules must stay provider-free:\n{}",
        violations.join("\n")
    );
}
