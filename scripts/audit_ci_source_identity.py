#!/usr/bin/env python3
"""Fail closed if main CI stops testing the exact source/head commit.

GitHub pull_request workflows default actions/checkout to refs/pull/<n>/merge.
Morn's repository-local engineering checkpoints are claims about the exact
feature/source HEAD, so CI must make that identity explicit and artifact names
must refer to the same source SHA.
"""

from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CI = ROOT / ".github" / "workflows" / "ci.yml"
SOURCE_EXPR = (
    "MORN_CI_SOURCE_SHA: ${{ github.event.pull_request.head.sha || github.sha }}"
)
CHECKOUT = "uses: actions/checkout@"


def step_block(lines: list[str], index: int) -> list[str]:
    start = index
    while start > 0 and not lines[start].startswith("      - "):
        start -= 1
    end = start + 1
    while end < len(lines) and not lines[end].startswith("      - "):
        end += 1
    return lines[start:end]


def main() -> int:
    text = CI.read_text(encoding="utf-8")
    lines = text.splitlines()
    violations: list[str] = []

    if SOURCE_EXPR not in text:
        violations.append("ci.yml must derive MORN_CI_SOURCE_SHA from PR head SHA with github.sha fallback")

    checkout_indices = [i for i, line in enumerate(lines) if CHECKOUT in line]
    if len(checkout_indices) != 5:
        violations.append(f"expected 5 CI checkout steps, found {len(checkout_indices)}")

    for index in checkout_indices:
        block = step_block(lines, index)
        joined = "\n".join(block)
        if "ref: ${{ env.MORN_CI_SOURCE_SHA }}" not in joined:
            violations.append(
                f"ci.yml:{index + 1}: checkout must pin ref to env.MORN_CI_SOURCE_SHA"
            )
        if "persist-credentials: false" not in joined:
            violations.append(
                f"ci.yml:{index + 1}: checkout must disable credential persistence"
            )

    required = (
        "name: Verify exact source checkout",
        'actual="$(git rev-parse HEAD)"',
        'test "$actual" = "$MORN_CI_SOURCE_SHA"',
        "name: morn-ui-reference-${{ env.MORN_CI_SOURCE_SHA }}",
        "name: morn-windows-reference-${{ env.MORN_CI_SOURCE_SHA }}",
    )
    for token in required:
        if token not in text:
            violations.append(f"ci.yml missing exact-source contract token: {token}")

    artifact_merge_sha = re.compile(r"^\s+name:\s+.*\$\{\{\s*github\.sha\s*\}\}", re.MULTILINE)
    if artifact_merge_sha.search(text):
        violations.append("CI artifact names must use MORN_CI_SOURCE_SHA, not pull_request merge github.sha")

    if violations:
        print("CI source-identity audit FAILED:")
        for violation in violations:
            print(f" - {violation}")
        return 1

    print("CI source-identity audit PASS:")
    print(" - pull_request jobs test the exact feature/source HEAD")
    print(" - all CI checkouts are source-pinned and credential-free")
    print(" - UI/desktop artifacts are named by the tested source SHA")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
