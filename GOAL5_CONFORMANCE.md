# GOAL5_CONFORMANCE.md

- ProviderConformance：registration/version/health/invoke/cancel/timeout/errors/audit/teardown/permissions。
- RuntimeConformance：start/stop/checkpoint/restore/signal/cancel/events/errors。
- ConnectorConformance：read/mapping/events/governed write/idempotency/retry/teardown。
- PluginConformance：manifest/compatibility/permissions/lifecycle/migrations/health。
- DomainPackConformance：no private Core imports；manifest/registrations/policies/work templates valid；install/enable/disable/uninstall；historical readability；zero Core modification。
- ArchitectureConformance：禁止依赖方向自动 CI fail。
