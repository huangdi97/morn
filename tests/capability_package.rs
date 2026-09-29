//! capability_package — Package / Registry / Provenance (Phase 13).
//!
//! Proves (Goal §17 / master §18):
//! - the digest is a REAL SHA-256 over the package content (ring): same
//!   content → same digest; different content → different digest;
//! - the local layout is digest-addressed and OCI-artifact-mappable
//!   (<root>/<digest>/{manifest.json,schemas/,tests/,docs/,...});
//! - provenance is honest: LOCAL UNSIGNED until a real signer runs — the
//!   signature field is null, never fabricated;
//! - package integrity: verify() recomputes the digest; tampering is
//!   detected;
//! - determinism: building the same package twice yields the same digest and
//!   the same layout.

use morn::studio::package::{
    sha256_hex, LocalPackageBuilder, LocalUnsignedProvenanceProvider, PackageInputs,
    ProvenanceProvider, ProvenanceStatus,
};

fn temp_root(label: &str) -> std::path::PathBuf {
    // Test isolation: a unique directory per test — concurrent tests must not
    // remove each other's packages (the layout is content-addressed under
    // the root).
    let dir = std::env::temp_dir().join(format!("morn_pkg_test_{label}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn g_sha256_is_a_real_digest() {
    // Real SHA-256: known-answer test (empty string digest is well-known).
    assert_eq!(
        sha256_hex(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    // Same content → same digest; different content → different digest.
    let a = sha256_hex(b"manifest-a");
    let a2 = sha256_hex(b"manifest-a");
    let b = sha256_hex(b"manifest-b");
    assert_eq!(a, a2);
    assert_ne!(a, b);
    assert_eq!(a.len(), 64, "SHA-256 hex is 64 chars");
}

#[test]
fn g_package_builds_content_addressed_layout() {
    let root = temp_root("layout");
    let package = LocalPackageBuilder::build(
        &root,
        &PackageInputs {
            name: "equipment-investigator".to_string(),
            version: "1.4.0".to_string(),
            manifest_json: r#"{"name":"equipment-investigator","version":"1.4.0"}"#.to_string(),
            schema_refs: vec!["schema://equipment-investigation-input/v1".to_string()],
            runtime_json: Some(r#"{"harnessCompatible":["dsh","pi"]}"#.to_string()),
            qualification_json: Some(
                r#"{"status":"PASSED","evaluator":"qualification-runner"}"#.to_string(),
            ),
            test_refs: vec!["tests/equipment.yaml".to_string()],
            provenance_json: Some(r#"{"sourceRef":"repo://..."}"#.to_string()),
            doc_refs: vec!["docs/README.md".to_string()],
        },
    )
    .expect("package builds");

    // The digest is a real SHA-256 over the content (64 hex chars).
    assert_eq!(package.digest.len(), 64);
    assert!(
        package.qualification_json.is_some(),
        "qualification evidence bundled"
    );

    // The OCI-mappable layout exists on disk under <root>/<digest>/.
    let pkg_dir = root.join(&package.digest);
    assert!(pkg_dir.join("manifest.json").is_file());
    assert!(pkg_dir.join("schemas").is_dir());
    assert!(pkg_dir.join("tests").is_dir());
    assert!(pkg_dir.join("docs").is_dir());
    assert!(pkg_dir.join("runtime.json").is_file());
    assert!(pkg_dir.join("qualification.json").is_file());
    assert!(pkg_dir.join("provenance.json").is_file());
}

#[test]
fn g_package_build_is_deterministic() {
    // Same content → same digest (content addressing).
    let root = temp_root("determinism");
    let manifest = r#"{"name":"demo","version":"1.0.0"}"#;
    let a = LocalPackageBuilder::build(
        &root,
        &PackageInputs {
            name: "demo".to_string(),
            version: "1.0.0".to_string(),
            manifest_json: manifest.to_string(),
            ..Default::default()
        },
    )
    .expect("build a");
    let b = LocalPackageBuilder::build(
        &root,
        &PackageInputs {
            name: "demo".to_string(),
            version: "1.0.0".to_string(),
            manifest_json: manifest.to_string(),
            ..Default::default()
        },
    )
    .expect("build b");
    assert_eq!(
        a.digest, b.digest,
        "content addressing: same content, same digest"
    );

    // Different content → different digest.
    let c = LocalPackageBuilder::build(
        &root,
        &PackageInputs {
            name: "demo".to_string(),
            version: "1.0.1".to_string(),
            manifest_json: manifest.to_string(),
            ..Default::default()
        },
    )
    .expect("build c");
    assert_ne!(a.digest, c.digest);
}

#[test]
fn g_package_verify_detects_tampering() {
    let root = temp_root("tamper");
    let package = LocalPackageBuilder::build(
        &root,
        &PackageInputs {
            name: "demo".to_string(),
            version: "1.0.0".to_string(),
            manifest_json: r#"{"name":"demo","version":"1.0.0"}"#.to_string(),
            ..Default::default()
        },
    )
    .expect("build");

    // Intact package verifies.
    LocalPackageBuilder::verify(&package).expect("intact package verifies");

    // Tampered manifest: digest mismatch is detected (supply-chain defense).
    let mut tampered = package.clone();
    tampered.manifest_json = r#"{"name":"demo","version":"9.9.9"}"#.to_string();
    let result = LocalPackageBuilder::verify(&tampered);
    assert!(result.is_err(), "tampering is detected");
    assert!(result.unwrap_err().contains("digest mismatch"));
}

#[test]
fn g_provenance_is_honestly_unsigned() {
    // SAFETY: Sigstore/SLSA cannot truly run locally yet — the provenance is
    // LOCAL UNSIGNED and the signature field is null, never fabricated.
    let provider = LocalUnsignedProvenanceProvider::new("morn-local-builder");
    let (status, record) = provider
        .record_provenance("sha256:abc")
        .expect("provenance");

    assert_eq!(status, ProvenanceStatus::LocalUnsigned);
    assert!(record.contains("LOCAL_UNSIGNED"));
    assert!(
        record.contains("\"signature\":null"),
        "signature is null — never faked"
    );
    assert!(record.contains("morn-local-builder"));

    // Empty digest: typed error.
    assert!(provider.record_provenance("").is_err());

    // The provider contract accepts any implementation (a real Sigstore
    // provider can replace it).
    let provider: &dyn ProvenanceProvider = &provider;
    assert_eq!(provider.provider_id(), "local-unsigned-provenance");
}
