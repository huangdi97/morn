#!/usr/bin/env python3
"""Fail closed on privileged GitHub workflow secret/lifecycle regressions.

This intentionally implements a small indentation-aware audit rather than a
full Actions evaluator. It checks repository-owned invariants that are easy to
review and stable across Actions syntax:
- protected live workflows must not expose secrets through job-level env;
- third-party actions in those workflows must be pinned to a full commit SHA;
- live DSH/Pi workflows must always clean one-shot runtime/session material.
"""

from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
WORKFLOW_DIR = ROOT / ".github" / "workflows"
LIVE = sorted(WORKFLOW_DIR.glob("live-*-evidence.yml"))
SHA_PIN = re.compile(r"^\s*-?\s*uses:\s*[^@\s]+@([0-9a-f]{40})\s*$")


def indentation(line: str) -> int:
    return len(line) - len(line.lstrip(" "))


def job_env_secret_violations(path: Path, lines: list[str]) -> list[str]:
    violations: list[str] = []
    i = 0
    while i < len(lines):
        line = lines[i]
        if line == "    env:":
            i += 1
            while i < len(lines):
                current = lines[i]
                if current.strip() and indentation(current) <= 4:
                    break
                if "${{ secrets." in current:
                    violations.append(
                        f"{path.relative_to(ROOT)}:{i + 1}: secret exposed through job-level env"
                    )
                i += 1
            continue
        i += 1
    return violations


def checkout_credential_violations(path: Path, lines: list[str]) -> list[str]:
    violations: list[str] = []
    for index, line in enumerate(lines):
        if "uses: actions/checkout@" not in line:
            continue

        # A checkout may be written either as "- uses: ..." or as a named step
        # with a sibling "uses:" key. Find the containing six-space list item
        # and inspect that whole step rather than treating the uses-line's
        # indentation as the step boundary.
        step_start = index
        while step_start > 0 and not lines[step_start].startswith("      - "):
            step_start -= 1
        step_end = step_start + 1
        while step_end < len(lines) and not lines[step_end].startswith("      - "):
            step_end += 1
        step = lines[step_start:step_end]

        if not any("persist-credentials: false" in current for current in step):
            violations.append(
                f"{path.relative_to(ROOT)}:{index + 1}: checkout must set persist-credentials: false"
            )
    return violations


def action_pin_violations(path: Path, lines: list[str]) -> list[str]:
    violations: list[str] = []
    for index, line in enumerate(lines, start=1):
        if "uses:" not in line:
            continue
        stripped = line.strip()
        if not stripped.startswith("uses:") and not stripped.startswith("- uses:"):
            continue
        match = SHA_PIN.match(line)
        if match is None:
            violations.append(
                f"{path.relative_to(ROOT)}:{index}: action must be pinned to a full 40-hex commit SHA"
            )
    return violations


def cross_step_secret_violations(path: Path, lines: list[str]) -> list[str]:
    sensitive_names = (
        "MORN_EVIDENCE_ISSUER",
        "MORN_BIOLAB_DATASET_PROVENANCE_ISSUER",
        "MORN_EXECUTION_ATTESTOR",
        "DEEPSEEK_API_KEY",
        "MORN_PI_PROVIDER_API_KEY",
        "MORN_REGISTRY_AUTH_JSON",
    )
    violations: list[str] = []
    for index, line in enumerate(lines, start=1):
        if "GITHUB_ENV" not in line:
            continue
        for name in sensitive_names:
            if name in line:
                violations.append(
                    f"{path.relative_to(ROOT)}:{index}: sensitive value {name} must not be persisted through GITHUB_ENV"
                )
    return violations


def require_cleanup(path: Path, text: str, tokens: tuple[str, ...]) -> list[str]:
    missing = [token for token in tokens if token not in text]
    if not missing:
        return []
    return [
        f"{path.relative_to(ROOT)}: missing mandatory always-cleanup marker(s): {', '.join(missing)}"
    ]


def main() -> int:
    violations: list[str] = []
    if not LIVE:
        violations.append("no protected live evidence workflows found")

    for path in LIVE:
        text = path.read_text(encoding="utf-8")
        lines = text.splitlines()
        violations.extend(job_env_secret_violations(path, lines))
        violations.extend(action_pin_violations(path, lines))
        violations.extend(checkout_credential_violations(path, lines))
        violations.extend(cross_step_secret_violations(path, lines))

    cleanup_requirements = {
        "live-dsh-evidence.yml": (
            "Remove live DSH runtime, session and attestation material",
            "if: always() && startsWith(inputs.operation, 'smoke-')",
            "MORN_DSH_WORKSPACE",
            "MORN_DSH_HOME",
            "morn-dsh-venv-",
            "MORN_EXECUTION_ATTESTATION_FILE",
        ),
        "live-pi-evidence.yml": (
            "Remove live Pi runtime, workspace and attestation material",
            "if: always()",
            "MORN_PI_WORKSPACE",
            "MORN_EXECUTION_ATTESTATION_FILE",
        ),
    }
    for name, tokens in cleanup_requirements.items():
        path = WORKFLOW_DIR / name
        if not path.exists():
            violations.append(f"{path.relative_to(ROOT)}: required workflow missing")
            continue
        violations.extend(require_cleanup(path, path.read_text(encoding="utf-8"), tokens))

    if violations:
        print("Privileged workflow audit FAILED:")
        for item in violations:
            print(f" - {item}")
        return 1

    print(f"Privileged workflow audit PASS: {len(LIVE)} live workflows")
    print(" - no job-level secret exposure")
    print(" - all external actions pinned to full commit SHAs")
    print(" - all checkouts disable credential persistence")
    print(" - protected issuer/credential values are not persisted through GITHUB_ENV")
    print(" - DSH/Pi one-shot runtime cleanup markers present")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
