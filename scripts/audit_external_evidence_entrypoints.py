#!/usr/bin/env python3
"""Fail closed when a protected live-evidence workflow loses its local entrypoints.

Each external blocker must keep a repository-local contract even while its real
evidence is unavailable:

- a manual dispatch entrypoint;
- an explicit plan mode plus a distinct live/verify mode;
- a plan path that never reads protected secrets;
- the deployment-owned environment and/or attested runner used by verify;
- evidence upload and always-cleanup of protected source material.

This audit is intentionally text/indentation based (no YAML dependency) and
locks the current reviewed matrix rather than guessing intent.
"""

from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
WORKFLOW_DIR = ROOT / ".github" / "workflows"

MATRIX = {
    "live-dsh-evidence.yml": {
        "operations": ["plan-sdk", "plan-acp", "smoke-sdk", "smoke-acp"],
        "plan_gate": "startsWith(inputs.operation, 'plan-')",
        "verify_gate": "startsWith(inputs.operation, 'smoke-')",
        "runner": "morn-attested-provider",
        "environment": "morn-live-provider",
        "cleanup": [
            "Remove live DSH runtime, session and attestation material",
            "MORN_DSH_HOME",
            "MORN_EXECUTION_ATTESTATION_FILE",
        ],
        "required_tokens": [
            "MORN_BUILD_IDENTITY_REF",
            "git://huangdi97/morn/${{ github.sha }}",
            "build-identity.json",
        ],
    },
    "live-pi-evidence.yml": {
        "operations": ["plan", "smoke"],
        "plan_gate": "inputs.operation == 'plan'",
        "verify_gate": "inputs.operation == 'smoke'",
        "runner": "morn-attested-provider",
        "environment": "morn-live-provider",
        "cleanup": [
            "Remove live Pi runtime, workspace and attestation material",
            "MORN_PI_WORKSPACE",
            "MORN_EXECUTION_ATTESTATION_FILE",
        ],
        "required_tokens": [
            "MORN_BUILD_IDENTITY_REF",
            "git://huangdi97/morn/${{ github.sha }}",
            "build-identity.json",
        ],
    },
    "live-supply-chain-evidence.yml": {
        "operations": ["plan", "verify"],
        "plan_gate": "inputs.operation == 'plan'",
        "verify_gate": "inputs.operation == 'verify'",
        "runner": "morn-attested-release",
        "environment": "morn-supply-chain-verification",
        "cleanup": ["Remove ephemeral registry credentials", "morn-registry-auth-"],
        "required_tokens": [
            "secrets.MORN_COSIGN_CERTIFICATE_IDENTITY",
            "secrets.MORN_COSIGN_OIDC_ISSUER",
            "MORN_SUPPLY_CHAIN_RELEASE_RUNNER_REF",
            "MORN_SUPPLY_CHAIN_COSIGN_DIGEST",
            "MORN_SUPPLY_CHAIN_VERIFIER_BUILD_REF",
            "git://huangdi97/morn/${{ github.sha }}",
            "slsaprovenance1",
            "Verify exact supply-chain verifier checkout",
            "morn-supply-chain://release-runner?value=",
            "morn-supply-chain://cosign-binary?value=",
            "morn-supply-chain://verifier-build?value=git://huangdi97/morn/",
            "proof_payloads",
            "cosign-signature-proof.json",
            "cosign-slsa-provenance-proof.json",
        ],
    },
    "live-biolab-evidence.yml": {
        "operations": ["plan", "verify"],
        "plan_gate": "inputs.operation == 'plan'",
        "verify_gate": "inputs.operation == 'verify'",
        "runner": "morn-attested-biolab",
        "environment": "morn-biolab-evidence",
        "cleanup": [
            "Remove protected provenance source",
            "MORN_BIOLAB_DATASET_PROVENANCE_FILE",
        ],
    },
    "live-customer-evidence.yml": {
        "operations": ["plan", "verify"],
        "plan_gate": "inputs.operation == 'plan'",
        "verify_gate": "inputs.operation == 'verify'",
        "runner": None,
        "environment": "morn-customer-evidence",
        "cleanup": ["Remove protected source claim", "MORN_EVIDENCE_CLAIM_FILE"],
        "required_tokens": [
            "build_identity_ref:",
            "INPUT_BUILD_IDENTITY_REF",
            "MORN_EVIDENCE_EXPECTED_BUILD_REF",
        ],
    },
}


def indentation(line: str) -> int:
    return len(line) - len(line.lstrip(" "))


def step_block(lines: list[str], index: int) -> list[str]:
    start = index
    while start > 0 and not lines[start].startswith("      - "):
        start -= 1
    end = start + 1
    while end < len(lines) and not lines[end].startswith("      - "):
        end += 1
    return lines[start:end]


def job_block(lines: list[str], index: int) -> list[str]:
    job_pattern = re.compile(r"^  [A-Za-z0-9_-]+:$")
    start = index
    while start > 0 and not job_pattern.match(lines[start]):
        start -= 1
    end = start + 1
    while end < len(lines) and not job_pattern.match(lines[end]):
        end += 1
    return lines[start:end]


def audit(name: str, expectations: dict[str, object]) -> list[str]:
    path = WORKFLOW_DIR / name
    if not path.exists():
        return [f"{name}: required workflow is missing"]
    lines = path.read_text(encoding="utf-8").splitlines()
    text = "\n".join(lines)
    violations: list[str] = []

    if "workflow_dispatch:" not in text:
        violations.append(f"{name}: workflow_dispatch entrypoint is missing")

    for operation in expectations["operations"]:
        if not re.search(rf"^\s+- {re.escape(operation)}\s*$", text, re.MULTILINE):
            violations.append(f"{name}: operation option {operation!r} is missing")

    plan_gate = str(expectations["plan_gate"])
    verify_gate = str(expectations["verify_gate"])
    if plan_gate not in text:
        violations.append(f"{name}: plan gate {plan_gate!r} is missing")
    if verify_gate not in text:
        violations.append(f"{name}: verify gate {verify_gate!r} is missing")
    if plan_gate == verify_gate:
        violations.append(f"{name}: plan and verify gates must be distinct")

    runner = expectations["runner"]
    if runner is not None and str(runner) not in text:
        violations.append(f"{name}: attested runner {runner!r} is missing")

    environment = expectations["environment"]
    if environment is not None and f"environment: {environment}" not in text:
        violations.append(f"{name}: protected environment {environment!r} is missing")

    if "upload-artifact@" not in text:
        violations.append(f"{name}: evidence upload step is missing")

    for token in expectations["cleanup"]:
        if str(token) not in text:
            violations.append(f"{name}: cleanup marker {token!r} is missing")

    for token in expectations.get("required_tokens", []):
        if str(token) not in text:
            violations.append(f"{name}: required contract token {token!r} is missing")

    plan_lines = [index for index, line in enumerate(lines) if plan_gate in line]
    if not plan_lines:
        violations.append(f"{name}: no executable plan path exists")
    for index in plan_lines:
        block = job_block(lines, index) if indentation(lines[index]) <= 4 else step_block(lines, index)
        if any("${{ secrets." in line for line in block):
            violations.append(
                f"{name}:{index + 1}: plan path must not read protected secrets"
            )

    return violations


def main() -> int:
    violations: list[str] = []
    for name, expectations in MATRIX.items():
        violations.extend(audit(name, expectations))

    if violations:
        print("External evidence entrypoint audit FAILED:")
        for violation in violations:
            print(f" - {violation}")
        return 1

    print(f"External evidence entrypoint audit PASS: {len(MATRIX)} live workflows")
    print(" - manual dispatch + explicit plan/verify modes")
    print(" - plan paths never read protected secrets")
    print(" - attested runner / protected environment retained")
    print(" - evidence upload and always-cleanup markers present")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
