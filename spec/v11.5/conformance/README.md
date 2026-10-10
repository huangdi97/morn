# Morn v11.5 black-box conformance

This directory defines **behavioral** test vectors for any implementation that
claims Morn v11.5 compatibility. Structural JSON Schema validation is necessary
but not sufficient.

A compatible runtime may be written in Rust, TypeScript, Java, Go, Python,
Wasm, or be hosted inside an existing enterprise platform. It does not need to
use Cordis, DSH, Pi, SQLite or the Morn reference server.

## Levels

### Protocol structural conformance

The implementation can parse/emit the published wire records and preserves
unknown/extension data where required by the relevant contract.

### Semantic conformance

The implementation passes every required vector in
`semantic-vectors.json`. These vectors test Morn's cross-record laws rather
than implementation details.

### Profile conformance

A composition additionally satisfies a concrete Domain Profile. For
`morn.factory.readonly@1.0.0`, this includes durable Work state,
source-of-truth binding, qualification, authority before side effects,
reconciliation, independent acceptance, provenance and the execution guarantee
floor.

### Evidence conformance

Claims are labelled by evidence class. Fixture/CI conformance cannot be
reported as site, shadow, production-write or commercial evidence.

## Black-box adapter contract

A third-party conformance harness only needs operations equivalent to:

```text
create_work(spec)
replace_work_spec(expected_resource_version, spec)
register_capability(manifest)
qualify_capability(evidence)
admit_capability(site, profile)
resolve_capabilities(request)
decide_authority(request)
create_binding(selection)
start_attempt(binding)
record_dispatch_result(result)
reconcile(attempt)
record_outcome(observation)
decide_acceptance(outcomes, evidence)
terminate_work(reason)
read_history(subject)
```

Names and transport are implementation-specific. The observable behavior is
what is tested.

## Non-goals

- identical model output;
- identical Agent session shape;
- identical database schema;
- identical Cordis plugin tree;
- identical UI;
- identical workflow engine.

Those are replaceable implementation choices.
