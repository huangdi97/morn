//! telemetry — Morn semantic attrs + CloudEvents + redaction (Phase 17).
//!
//! Proves (Goal §21 / master §23):
//! - the Morn OTel attribute keys are the canonical semantic attrs (stable
//!   conformance surface, morn.* namespaced);
//! - CloudEvents envelopes validate the required fields and carry the Morn
//!   extension attributes;
//! - the trace chain Work → resolve → bind → authorize → dispatch →
//!   receipt/reconcile → outcome → acceptance is expressible as a SpanAttrs
//!   evolution;
//! - logs redact secrets — never recorded in full.

use morn::telemetry::{
    redact_for_log, CloudEvent, SpanAttrs, ALL_ATTR_KEYS, ATTR_ATTEMPT_ID, ATTR_BINDING_ID,
    ATTR_CAPABILITY_DIGEST, ATTR_OUTCOME_ID, ATTR_WORK_ID, ATTR_WORK_TYPE,
};

#[test]
fn g_morn_attr_keys_are_stable_and_namespaced() {
    // The canonical attribute set: 11 keys, all morn.* namespaced, work.id
    // and work.type present.
    assert_eq!(ALL_ATTR_KEYS.len(), 11);
    assert_eq!(ALL_ATTR_KEYS[0], ATTR_WORK_ID);
    assert_eq!(ALL_ATTR_KEYS[1], ATTR_WORK_TYPE);
    for key in ALL_ATTR_KEYS {
        assert!(
            key.starts_with("morn."),
            "attribute keys are namespaced: {key}"
        );
    }
}

#[test]
fn g_cloudevents_envelope_validates_required_fields() {
    // A valid envelope passes validation and carries the Morn extensions.
    let event = CloudEvent::new(
        "evt-23898",
        "morn://plant-a/cmms",
        "io.morn.action.receipt.v1",
        Some("work/work-cnc17"),
        Some(serde_json::json!({ "attempt_id": "attempt-cnc17", "external_ref": "MO-88273" })),
    )
    .with_span_attrs(&SpanAttrs {
        work_id: Some("work-cnc17".to_string()),
        binding_id: Some("binding-cnc17".to_string()),
        attempt_id: Some("attempt-cnc17".to_string()),
        profile_id: Some("morn.factory.readonly".to_string()),
        ..Default::default()
    });

    event.validate().expect("valid envelope");
    assert_eq!(event.specversion, "1.0");
    assert_eq!(event.mornworkid.as_deref(), Some("work-cnc17"));
    assert_eq!(event.mornbindingid.as_deref(), Some("binding-cnc17"));
    assert_eq!(event.mornattemptid.as_deref(), Some("attempt-cnc17"));
    assert_eq!(event.mornprofile.as_deref(), Some("morn.factory.readonly"));

    // Missing required fields: typed errors.
    let empty = CloudEvent::new("", "", "", None, None);
    assert!(empty.validate().is_err());
    let no_type = CloudEvent::new("e", "morn://x", "", None, None);
    assert!(no_type.validate().is_err());

    // The envelope round-trips through serde (cross-component format).
    let json = serde_json::to_string(&event).unwrap();
    let back: CloudEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(back, event);
}

#[test]
fn g_trace_chain_is_expressible_as_span_attrs() {
    // Work created → resolve → bind → authorize → dispatch → receipt →
    // outcome → acceptance: each step enriches the attribute set; the work.id
    // is the correlation id across the whole chain.
    let mut attrs = SpanAttrs::default();
    attrs.work_id = Some("work-cnc17".to_string());
    attrs.work_type = Some("factory.equipment-exception".to_string());

    // resolve
    attrs.capability_id = Some("equipment-investigator".to_string());
    attrs.capability_digest = Some("sha256:equipment-investigator".to_string());
    // bind
    attrs.binding_id = Some("binding-cnc17".to_string());
    attrs.profile_id = Some("morn.factory.readonly".to_string());
    attrs.provider_id = Some("fake-dsh".to_string());
    attrs.site_id = Some("plant-a".to_string());
    // authorize
    attrs.authority_decision_id = Some("authz-cnc17".to_string());
    // dispatch
    attrs.attempt_id = Some("attempt-cnc17".to_string());
    // outcome
    attrs.outcome_id = Some("outcome-cnc17".to_string());

    let pairs = attrs.to_pairs();
    // Every canonical key has a value by the end of the chain.
    assert_eq!(pairs.len(), 11, "the full chain fills every attribute");
    // work.id correlates across the chain.
    let keys: Vec<&str> = pairs.iter().map(|(k, _)| k.as_str()).collect();
    assert!(keys.contains(&ATTR_WORK_ID));
    assert!(keys.contains(&ATTR_ATTEMPT_ID));
    assert!(keys.contains(&ATTR_BINDING_ID));
    assert!(keys.contains(&ATTR_CAPABILITY_DIGEST));
    assert!(keys.contains(&ATTR_OUTCOME_ID));

    // Partial state: only the set fields are rendered.
    let partial = SpanAttrs {
        work_id: Some("w-1".to_string()),
        ..Default::default()
    };
    assert_eq!(partial.to_pairs().len(), 1);
}

#[test]
fn g_logs_redact_secrets() {
    // SAFETY: secrets are never recorded in full — keys named like a secret
    // are [REDACTED], nested payloads are walked, non-secret data survives.
    let payload = serde_json::json!({
        "work_id": "work-cnc17",
        "status": "VERIFIED",
        "password": "hunter2",
        "api_key": "sk-secret-123",
        "credential": "cred-9",
        "nested": {
            "token": "tok-7",
            "detail": "order committed",
            "note": "no secret here"
        },
        "items": [ { "password": "p1" }, { "ok": true } ]
    });
    let redacted = redact_for_log(&payload);

    assert_eq!(
        redacted["work_id"], "work-cnc17",
        "non-secret data survives"
    );
    assert_eq!(redacted["status"], "VERIFIED");
    assert_eq!(redacted["password"], "[REDACTED]");
    assert_eq!(redacted["api_key"], "[REDACTED]");
    assert_eq!(redacted["credential"], "[REDACTED]");
    assert_eq!(redacted["nested"]["token"], "[REDACTED]");
    assert_eq!(redacted["nested"]["detail"], "order committed");
    assert_eq!(redacted["nested"]["note"], "no secret here");
    assert_eq!(redacted["items"][0]["password"], "[REDACTED]");
    assert_eq!(redacted["items"][1]["ok"], serde_json::json!(true));

    // The full secret value never appears anywhere in the redacted JSON.
    let redacted_str = redacted.to_string();
    assert!(!redacted_str.contains("hunter2"));
    assert!(!redacted_str.contains("sk-secret-123"));
    assert!(!redacted_str.contains("tok-7"));
}
