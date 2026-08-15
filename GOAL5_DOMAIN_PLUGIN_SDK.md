# GOAL5_DOMAIN_PLUGIN_SDK.md

## DomainDefinition
Domain Pack 可声明：domain/version/sdk/dependencies/permissions/ontology/object types/relation types/events/actions/artifacts/outcomes/roles/work templates/policies/capabilities/connector requirements/evaluations/UI extensions。

约束：只依赖 public Morn SDK；不得 patch Core source；不得 raw write Core DB；不得绕过 permissions；不得拥有 identity/workspace truth。

## Domain Lifecycle
validate compatibility → build → install → enable → disable → upgrade → uninstall。Upgrade 要 preflight/dry-run/snapshot/apply/verify。Uninstall 保留历史 canonical records/provenance/version descriptor。

## Plugin Types
capability-provider / harness-provider / runtime-provider / connector-provider / domain-pack / evaluation-pack / simulation-pack / ui-extension / cli-extension。

Plugin manifest 至少：id/version/type/core_compat/sdk_compat/dependencies/permissions/entrypoints/config schema/secrets/migrations/health checks。

权限声明不等于自动授权，runtime policy 才是最终权威。
