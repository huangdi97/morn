# GOAL5_UI_CLI_SPEC.md

## Zero-Domain UI

Workbench：Mission/Work/WorkGraph/Workcell/Artifact/Decision/Outcome/Approval/Attention/Execution Timeline。

Studio：WorkPackage/Workflow/Role/Actor/Capability/Provider/Connector/Domain Pack/Solution/Evaluation/Simulation builders。

Console：Identity/Workspace/Policy/Secrets health/Nodes/Runtimes/Providers/Connectors/Plugins/Domain Packs/Deployments/Migrations/Audit/Incidents/Versions。

Hub：package registry/provider packages/domain packs/templates/compatibility/trust/version/dependencies。

Enable reference domain 后，UI 只能通过 extension point 注入；disable/uninstall 后注入内容消失，Core routes 稳定。

## CLI
优先整合现有 CLI：
`morn doctor`, `status`, `migrate`, `node list/doctor`, `provider list`, `connector list`, `plugin list`, `domain init/validate/build/install/enable/disable/upgrade/uninstall/inspect/diff`, `package inspect`, `compatibility check`, `conformance run`, `test core`。

CLI 必须调用 canonical services，不直接操作 DB。
