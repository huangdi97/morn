# BLOCKERS.md

只记录 **无法通过继续编码/测试在当前环境解决** 的真实阻塞。

## Current blocker index — 2026-10-09

This section is authoritative for the current v11.5 convergence branch. Older
entries below are retained as historical evidence and must not be read as the
current implementation state.

- **B-001 / DSH live evidence — EXTERNAL_BLOCKED.** The official DSH
  distribution and Morn real SDK/JSON-RPC adapter now exist. Remaining evidence
  requires an authorized runtime, exact model route/credentials, a real
  runtime-attested execution environment, and a live settled-turn smoke.
  Fixture/wire tests do not satisfy this gate.
- **B-002 / Pi live evidence — EXTERNAL_BLOCKED.** Morn has a real Pi JSONL RPC
  adapter; a live installed Pi runtime/model/credential path and an authorized
  runtime-attested execution environment are still required for a real smoke.
- **B-003 / execution containment — EXTERNAL_BLOCKED for live evidence.** Morn
  has provider-neutral environment contracts and an attestation-backed external
  environment adapter. A real container/microVM/Kubernetes/customer-sandbox
  attestation, with exact environment identity and required guarantee vector,
  must come from the deployment platform. Morn does not infer containment from
  a Docker binary or a caller-supplied label.
- **G4-B-002 / real BioLab dataset — EXTERNAL_BLOCKED.** A lawful,
  provenance-bearing real dataset is still required.
- **G12 / customer-site acceptance — EXTERNAL_BLOCKED.** Real source-system
  access, site IAM/policy, customer evidence and independent acceptance have not
  been supplied.
- **Production/physical write — NOT AUTHORIZED.** No local fixture, Harness
  success, Provider health lease, or CI run upgrades this state.

Current code therefore may reach **ALL_LOCAL_GATES_PASS** while these gates stay
external. It must not claim authenticated provider, real-site, customer-value,
production-write or physical-control evidence without those sources.

## Historical evidence retained below

> **Superseded:** the August B-001 record below captures what was observed in that older environment.
> It is not the current blocker definition. The authoritative 2026-10-09 blocker index above and
> the 2026-10-08 revalidation section below supersede claims such as “no official DSH distribution”
> or “DshMode::Real is only a stub.” Do not use the historical reproduction as current product truth.

### Historical B-001 — August environment could not complete a real DeepSeek Harness smoke
Status: Superseded historical evidence
Category: Credential / ExternalService

What was blocked at that time:
The August environment could not locate the later official DeepSeek Harness distribution or perform a credentialed real smoke. The commands and errors below are retained only as provenance for that historical observation.

Why Codex cannot resolve locally:
1. 无官方 DSH 二进制/包；唯一同名的 PyPI `deepseek-harness` 0.2.0 是第三方 OpenAI 兼容客户端，需要真实 DeepSeek API key（credential）与真实模型访问，当前无凭据；
2. 需要真实 secret/credential 属于工程包定义的“请求人类”条件。

Reproduction:
```text
OS: Windows (x86_64-pc-windows-msvc)
Rust: 1.97.1   Node: 22.15.0   Python: 3.12.7

# 1) 探测官方安装途径
Get-Command dsh -> not found
npm search deepseek-harness -> 仅第三方包（如 @banlan/inkstone 对话历史查看器）
python -m pip index versions deepseek-harness -> deepseek-harness (0.2.0)

# 2) 下载并检查 PyPI 包（第三方 OpenAI 兼容客户端）
pip download deepseek-harness==0.2.0 --no-deps
unzip -> deepseek_harness/{cache,client,exceptions,normalize,reasoning,summarize,tool_calls}.py
METADATA -> "Protocol-aware client for DeepSeek V4-Pro / V4-Flash ... wraps openai.OpenAI"

# 3) 尝试真实 smoke（无凭据）
from deepseek_harness import DeepSeekHarness
c = DeepSeekHarness(disable_thinking_by_default=True)
c.chat(model="deepseek-v4-pro", messages=[...])
-> OpenAIError: The api_key client option must be set either by passing api_key to the client
   or by setting the OPENAI_API_KEY environment variable
```

Environment:
- OS: Windows
- runtime: Python 3.12.7 / Node 22.15.0 / Rust 1.97.1 (MSVC)
- 无 DSH 官方安装物；无 DeepSeek API 凭据

Attempts:
- 探测 dsh CLI（无）
- npm 搜索官方发行（无官方包）
- pip 下载唯一同名包并检查元数据（第三方 OpenAI 兼容客户端）
- 无凭据真实 smoke -> `OpenAIError: api_key ... must be set`

Safe workaround:
- `DeepSeekHarnessProvider`（`crates/morn-harness/src/provider.rs`）实现完整 provider 边界；
- `DshMode::Fixture` 通过同一 provider contract suite（`crates/morn-harness/tests/contract_tests.rs`），作为第二条 provider path；
- `DshMode::Real` 返回结构化 `Error::External`，不伪造成功。

Impact on acceptance:
- A5/DSH B1（Morn 侧 contract）通过；
- DSH B2（真实 smoke）不通过，按规则不允许伪造 → 记录为本 blocker。

Resolution:
需要（a）官方 DSH 安装途径，或（b）真实 DeepSeek API 凭据。Morn 侧 contract tests 在两种情况都保持通过。
## Goal 2 Update (2026-08-15)

无新增 blocker。Goal 1 的 B-001（真实 DeepSeek Harness smoke）仍然 Active：Goal 2 不依赖真实 DSH（compiler/durable/
replay/simulation/shadow/BioLab 全部使用 MornNative + fixture），Morn 侧 contract tests 保持通过。
## Goal 3 Update (2026-08-15)

无新增 blocker。Goal 1 的 B-001（真实 DeepSeek Harness smoke）仍然 Active；Goal 3 的 certification/
managed work/replacement 全部使用本地 deterministic/evaluation 证据，不依赖真实 DSH。
### G4-B-002 — 真实 BioLab 数据 pilot FULL blocked（无合法真实 dataset）
Status: Active
Category: ExternalService / Data

What is blocked:
`Dataset → Reviewed Scientific Claim` 的真实数据 pilot（pre-run predictions → managed work → actual → error →
calibration）无法在无合法真实 BioLab dataset 时执行。禁止捏造“真实数据”。

Why Codex cannot resolve locally:
仓库/环境没有用户提供的真实 dataset，也没有已核验许可证的公开 BioLab dataset 可直接使用；下载并核验大型公共
数据集（如 GEO 单细胞数据）超出本地自动化边界且涉及许可核验。

Reproduction:
```text
morn-opint::RealPilotService::run_pipeline(...)  -> Err("no lawful real BioLab dataset registered; FULL pilot is BLOCKED")
```

Safe workaround（已完成）:
- `PilotManifest`（source/license/checksum 必填，synthetic 拒绝）+ pipeline contract + 校验测试。
- OperationalEpisode / OutcomeDataset / Predictors / Calibration / Drift 全部以权威 Morn records（fixture 受控运行）
  验证闭环，CORE COMPLETE。

Impact on acceptance:
GOAL4 Pilot FULL 项保持 blocked；CORE 项（adapter/manifest/provenance/validation）全部通过。

Resolution:
需要用户提供合法真实 BioLab dataset（或明确授权可下载的公开数据 + 许可证）。## Goal 3 Re-verification (2026-08-16)

无新增 blocker。B-001（真实 DeepSeek Harness smoke，需官方安装物或真实凭据）与 G4-B-002（真实 BioLab 数据
pilot，需合法真实 dataset）保持 Active。Goal 3 全部本地可完成项 re-verified green：
`scripts/run_all.ps1` exit 0（208 Rust tests / 0 ignored；frontend 2；UI smoke 全部 OK；demo smoke OK）。

## Goal 5 Update (2026-08-16)

无新增 blocker。B-001（真实 DeepSeek Harness smoke）与 G4-B-002（真实 BioLab 数据 pilot）保持 Active；
二者均不阻塞 Goal 5 任何本地可完成项（Provider/Connector/Node/Distributed 全部以 fixture/conformance 证明）。
`CORE COMPLETE = YES`（唯一非本地项即上述两个 external blocker，与 Core 本体无关）。


## B-001 2026-10-08 revalidation — official DSH is installable

The earlier August record above is preserved as **historical evidence** and must not
be repeated as a current claim. The official DeepSeek Harness now has public
source code and an official npm launcher:

- https://github.com/deepseek-ai/deepseek-harness
- https://www.deepseek.com/harness/en/
- `npx @deepseek-ai/dsh web` (interactive Web UI, not Morn's provider API).
- The official `@deepseek-ai/dsh-acp` supports automation over ACP/JSON-RPC
  stdio, with `pnpm dsh --profile acp` from a built source checkout:
  https://github.com/deepseek-ai/deepseek-harness/blob/master/packages/acp/acp/README.md

Current truth:
- **OFFICIAL_INSTALL_DISTRIBUTION = AVAILABLE** (the August missing-package
  premise is no longer valid).
- **MORN_DSH_REAL_SDK_ADAPTER = IMPLEMENTED_CODE_PENDING_EXACT_HEAD_CI**:
  `crates/morn-harness/src/provider.rs` now accepts an explicit isolated
  `DshSdkConfig`, initializes the official SDK JSON-RPC wire, owns a
  receipt-to-idle turn, and records executor evidence without promoting it to
  Work truth. A protocol fixture test exercises the wire without credentials.
  The unconfigured `DshMode::Real` path still fails closed.
- **AUTHENTICATED_REAL_DSH_SESSION = NOT_VERIFIED**: official credentials and
  a sandboxed external runtime have not been supplied or exercised.
- **MORN_WORK_OUTCOME_ACCEPTANCE = NOT_PROVEN_BY_DSH**: no harness status
  can independently assert a customer's real observed/accepted outcome.

B-001 now means **authenticated official-runtime smoke / live provider evidence blocked**,
not "no official package exists" and not "the Morn-side SDK adapter is absent." The adapter path and permission controls are
specified in
`docs/research/V11_5_DSH_ACP_INTEGRATION_CONVERGENCE_2026-10-08.md`.
Never treat a successful `dsh web` launch as completion of the Morn
HarnessProvider contract or as production authorization.
