# Goal4 Test Plan

A Goal1-3 regression：existing tests、Goal3 E2E、frontend/Tauri/Playwright。

B Persistence：fresh migration、upgrade DB、restart hydration、workspace isolation、immutable history、idempotency、version conflict、API parity。

C Rollback：valid/incompatible/unauthorized/verification failure；E3 history preserved。

D Provider：structured parse、invalid/policy violation rejected、provider failure、no secret log、real smoke only when configured。

E Episode/Data：six-record mapping、provenance、authoritative labels、snapshot reproducibility、temporal split、duplicate/future leakage、workspace boundary。

F Predictors：每类 train/baseline→evaluate→serialize/version→load→predict→uncertainty→context mismatch。

G Calibration：prediction stored before actual；actual cannot rewrite prediction；calibration/drift update。

H Compiler/Evolution：multiple candidates、predictions attached、low confidence no auto-select、actual delta feedback。

I Real Pilot：有真实数据则 provenance/checksum/E2E/actual/error；无数据则 adapter tests + FULL BLOCKED。

J UI：persistence health、registry、candidate compare、prediction-vs-actual、rollback、external-blocker truthfulness。
