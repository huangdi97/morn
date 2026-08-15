# BLOCKERS.md

只记录 **无法通过继续编码/测试在当前环境解决** 的真实阻塞。

## Active

### B-001 — 真实 DeepSeek Harness smoke 无法在当前环境完成
Status: Active
Category: Credential / ExternalService

What is blocked:
官方 DeepSeek Harness（DSH，agent harness，Developer Preview）在当前环境没有官方可安装发行物；无法执行“真实 DSH 启动 + 至少一个真实 smoke”。Morn 侧 `DeepSeekHarnessProvider` 边界与 provider contract（fixture）已完成并通过，但这不是“真实 DSH 已集成”。

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
需要用户提供合法真实 BioLab dataset（或明确授权可下载的公开数据 + 许可证）。