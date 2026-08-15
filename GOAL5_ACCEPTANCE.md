# GOAL5_ACCEPTANCE.md

## A Baseline
- [ ] Goal1–4 regressions green
- [ ] reality audit complete

## B Domain Neutrality
- [ ] domain audit
- [ ] no Core→domain dependency
- [ ] Core zero-domain build/start/work
- [ ] generic UI zero-domain

## C Reference Extraction
- [ ] BioLab outside Core
- [ ] public Domain SDK only
- [ ] conformance passes
- [ ] Core passes with pack absent
- [ ] history preserved after disable/uninstall

## D Stable Kernel
- [ ] semantic contract v1
- [ ] API/schema versions
- [ ] compatibility/deprecation policy
- [ ] contract tests

## E Capability/Provider
- [ ] generic capability types
- [ ] no AI-only assumptions
- [ ] HarnessProvider/RuntimeProvider
- [ ] >=2 implementations/fixtures
- [ ] conformance/health/fallback/permission

## F Connector
- [ ] generic contract/external refs/mapping/sync cursor
- [ ] governed writes
- [ ] retry/idempotency
- [ ] conformance fixture

## G Process Intelligence
- [ ] generic trace/observed graph
- [ ] bottleneck/wait/rework/loop/handoff primitives
- [ ] no domain dependency

## H Node/Distributed
- [ ] node identity/registration/capabilities/resources/health/lease
- [ ] claim/heartbeat/failover/checkpoint transfer/remote execution/signal/dedupe
- [ ] real local 2-node E2E

## I Deployment
- [ ] topology/placement/runtime-storage-secret-policy bindings
- [ ] validation/upgrade/rollback strategy schema

## J Domain SDK / Pack
- [ ] ontology/type/action/event/artifact/outcome/role/work/policy/capability/connector/evaluation/UI extension contracts
- [ ] init/validate/build/install/enable/disable/upgrade/uninstall/inspect/diff

## K Plugin
- [ ] manifest/deps/permissions/compat/migrations/health/lifecycle
- [ ] Core policy cannot be bypassed

## L Product UI
- [ ] zero-domain Workbench/Studio/Console/Hub
- [ ] dynamic domain UI extension
- [ ] no static fake must-pass data

## M CLI
- [ ] doctor/status/node/provider/connector/plugin/domain/package/compat/conformance

## N Compatibility/Migration
- [ ] matrix/preflight/fresh+upgrade/failure/restore plan/no silent destructive migration

## O Security
- [ ] workspace isolation/secret redaction/provider-connector permissions/node auth/package permissions/audit/injection/path traversal

## P Reliability
- [ ] process/node/provider/connector/DB/migration/plugin/duplicate event/action failure tests
- [ ] explicit recover/block/escalate/compensate

## Q Conformance
- [ ] Provider/Runtime/Connector/Plugin/DomainPack/Architecture conformance

## R Final
- [ ] Pure Core v1.0 RC E2E
- [ ] Reference Pack E2E
- [ ] backend regression
- [ ] frontend typecheck/lint/test/build
- [ ] desktop build where supported
- [ ] UI smoke
- [ ] security + chaos
- [ ] reports/goal5_final_report.md
- [ ] CORE COMPLETE explicitly stated

禁止：移动文件不修依赖、假 distributed、假 connector/provider、static fake UI、ignored tests、弱化断言、todo!/unimplemented!、BioLab 改名冒充 generic、插件 raw DB、Connector 绕 Action Gateway、uninstall 删除历史、fake external integration。
