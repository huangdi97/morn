# Final Acceptance — Morn v1 GA + GitHub Takeover

## Local quality
- [ ] Goal1–5 regression green
- [ ] MUST_FIX_NOW = 0
- [ ] architecture audit clean
- [ ] code quality pass
- [ ] migrations/API pass
- [ ] provider/runtime/connector conformance
- [ ] distributed two-node proof
- [ ] domain/plugin/package conformance
- [ ] security pass
- [ ] chaos pass
- [ ] performance baseline recorded

## UI
- [ ] Workbench
- [ ] Studio
- [ ] Console
- [ ] Hub
- [ ] zero-domain
- [ ] loading/error/empty/blocked
- [ ] frontend build
- [ ] Playwright
- [ ] Tauri where supported

## Docs/CI/Package
- [ ] README/Quickstart
- [ ] Architecture
- [ ] SDK docs
- [ ] Deployment/Upgrade/Troubleshooting
- [ ] CHANGELOG/Release Notes
- [ ] GitHub Actions
- [ ] release build/package

## Local Git
- [ ] no secrets/local db/log/cache
- [ ] gitignore correct
- [ ] logical commits
- [ ] final local tests after commits
- [ ] GIT_STATUS=CLEAN
- [ ] LOCAL_FINAL_COMMIT recorded

## GitHub legacy
- [ ] old default branch identified
- [ ] legacy/pre-rewrite points to old tip
- [ ] legacy branch pushed
- [ ] legacy annotated tag pushed
- [ ] legacy commit recorded

## New GitHub
- [ ] morn-v1 push success
- [ ] morn-v1 == LOCAL_FINAL_COMMIT
- [ ] fresh fetch before main takeover
- [ ] remote old main unchanged
- [ ] main takeover uses force-with-lease only
- [ ] origin/main == LOCAL_FINAL_COMMIT after push
- [ ] no unrelated-history merge commit

## Final report
- [ ] reports/morn_v1_ga_final_report.md
- [ ] MORN_V1_GA_LOCAL_COMPLETE=YES
- [ ] GITHUB_MIGRATION_COMPLETE=YES or exact external blocker
