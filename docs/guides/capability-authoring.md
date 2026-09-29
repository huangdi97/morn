# Capability Authoring Guide（Morn v11.5-R1）

Capability = Implementation + CapabilityManifest + InterfaceContract + ExecutionRequirements + Qualification + AuthorityEnvelope + Provenance。

## 四种能力状态必须分离

```text
Declared  → Observed  → Qualified  → Released  → Site-Admitted
```

一个 Agent 自己写"我会排产"不等于排产能力已合格。注册 ≠ qualified ≠ admitted。

## CapabilityManifest

参考 `src/work/capability.rs`：

```yaml
apiVersion: capability.morn.io/v1alpha1
kind: Capability
metadata: { name: equipment-investigator, version: 1.4.0, digest: sha256:... }
spec:
  provides: [equipment.anomaly.investigate]
  sideEffects: { level: none }
  execution: { harnessCompatible: [dsh, pi], minimumIsolation: container }
  authority: { maximum: [historian.read, cmms.read] }
  economics: { expectedLatencyP95: 45s }
  provenance: { sourceRef: repo://... }
```

## 编译（Artifact2Capability）

两个真实 compiler（`src/studio/artifact2capability.rs`）：OpenAPI2Capability（真实 OpenAPI JSON 结构解析）+ Repo2Capability（真实文件系统扫描 entrypoint）。确定性——无 LLM。

Pipeline：Artifact discovery → extraction → candidate manifest → wrapper → environment → tests → evaluation → candidate qualification → package。

失败是合法结构化状态：RESOURCE_ONLY / BLOCKED_MISSING_ARTIFACT / BLOCKED_ENVIRONMENT / WRAPPED_BUT_UNVERIFIED / NON_GENERALIZABLE / FAILED_TEST。

绝不能：读文档 → LLM 生成一个 tool → RELEASED。

## Qualification

至少提供：deterministic test suite refs、environment digest、input scope、expected outputs、failure modes、cost/latency evidence、evaluator identity（被评对象不能自己改）。

## Release

Package 用 content-addressed digest（真实 SHA-256，`src/studio/package.rs`）；qualification record 通过 strict-profile gate 后才能进关键 binding。Provenance 诚实 LOCAL UNSIGNED（Sigstore/SLSA contract 已保留）。
