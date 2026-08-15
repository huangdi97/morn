# GOAL5_TEST_PLAN.md

1. Goal1–4 full regression first/last。
2. Domain neutrality：zero-pack build/test/start；forbidden import guard；no concrete domain enum in Core。
3. Reference extraction：validate/install/enable/E2E/disable/uninstall/history readable。
4. Kernel contracts：snapshot/version/incompatibility/deprecation。
5. Capability/Provider：2 implementations/fallback/timeout/cancel/health/error/permission。
6. Connector：read/event/mapping/governed write/retry/duplicate/rate-limit/teardown。
7. Process Intelligence：events→trace→observed graph；loop/rework/wait/handoff detection；no domain ontology。
8. Node/Distributed：2+ local nodes；lease/heartbeat/loss/failover/checkpoint transfer/duplicate event/idempotent action。
9. Deployment：desktop/server/team/edge topology valid；incompatible placement rejected。
10. Domain SDK：minimal hello-domain fixture init/validate/build/install/register generic object/work/action，无 Core edit。
11. Plugin：install/enable/disable/upgrade/uninstall；incompatible/permission/broken plugin isolation。
12. CLI：doctor/status/domain lifecycle/plugin/provider/connector/compat/conformance。
13. Security：执行安全矩阵。
14. Chaos：执行 failure injection 矩阵。
15. UI：zero-domain 4 surfaces；install reference domain 注入；disable/uninstall 后 UI 健康。
16. Pure Core E2E：generic work→workcell→execute→approval→artifact→outcome→restart→node failover→audit。
17. Reference E2E：install reference→conformance path→uninstall→Core unaffected。
