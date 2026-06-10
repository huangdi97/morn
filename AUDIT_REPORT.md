# Morn Core Architecture + COO/Supervisor Audit

> Generated: 2026-06-10
> Codebase: 272 .rs files, 45737 lines Rust, 1006 tests
> Verifying GAP_ANALYSIS.md (2026-06-07) claims against CURRENT code

---

## SECTION 2: Core Architecture Gaps

### 1. Dual-LLM 副 LLM 未实际接入 (was 🟡)
**Status: 🟡 Still Partial**

- **Engine has structure**: `secondary_llm: Option<LlmJudgeFn>` and `run_secondary_check()` calls judge closure or falls back to pattern matching
- **No production caller connects a real LLM**: All 4 callers use `DualLlmGuard::new(None, None)` — `with_llm_checks()` only used in tests
- **Evidence**:
  ```
  src/console/security.rs:87:  DualLlmGuard::new(None, None)
  src/console/mod.rs:357:      DualLlmGuard::new(None, None)
  src/console/mod.rs:373:      DualLlmGuard::new(None, None)
  ```
- The secondary check exists as a code path but there's no actual LLM API call wired in for production use

### 2. 独立线程池 (was ❌)
**Status: 🟡 Still Partial**

- `src/core/thread_pool.rs` — **ThreadPool exists** with:
  - `ThreadPoolConfig` with coo_threads/execution_threads/channel_threads/tray_threads
  - `start()` spawns real OS threads per pool
  - `stop()` with JoinHandle tracking
  - `TaskPool` with `execute()` → `tokio::task::spawn_blocking`
- **BUT**: The spawned threads just sleep-loop — no actual work is dispatched to them
- **TaskPool only handles PipelineExec** actions, nothing else
- The infrastructure is structurally present but functionally minimal (sleeping threads)

### 3. 子进程隔离 (was ❌)
**Status: ✅ CLOSED — Now implemented**

- `src/core/task_engine/child_process.rs` — **full implementation**:
  - `ChildProcess::spawn(task_json, timeout_secs)` — spawns same binary with `--execute-task`
  - Piped stdin/stdout/stderr
  - `wait()` with timeout and kill on timeout
  - `kill()` and `is_alive()` methods
- 84 lines, real subprocess isolation via `std::process::Command`

### 4. 渠道线程池 (was ❌)
**Status: ❌ Still Missing**

- Channel adapter (`src/channel/adapter.rs`) **never references ThreadPool or any thread pool**
- `ThreadPool` has `channel_threads: 2` in config but channels are not wired to it
- Zero grep matches: `grep -c thread_pool\|ThreadPool src/channel/*.rs` = 0
- No channel-worker dispatch infrastructure exists

---

## SECTION 4: COO/Supervisor Gaps

### 1. 用户强制指定层级 (was ❌)
**Status: ✅ CLOSED**

- **`override_decision(level, scope)`** at `src/core/supervisor/mod.rs:126`
- **`DecisionOverride`** struct with `NextTurn` and `Session` scopes (`types.rs:154`)
- **`DecisionOverride::parse_prefixed()`** for inline syntax: `"L4: hello"` → override to L4Team
- **G22 override keywords** in `decide_level()`: "直接回答" → L1, "use data team" → L4
- Used in `execute_chat()` and `decide_with_rules()`
- Tested: `tests.rs:90-97` (session override), `decision.rs:291-299` (override in decide_with_rules)

### 2. 三种工作模式真正差异 (Safe/Auto were ❌)
**Status: 🟡 Still Partial**

- **Mode enum** has all 3 values: `Proactive`, `Safe`, `Automated` (`types.rs:122-126`)
- **Real behavioral differences exist but are subtle**:
  - `dispatch.rs:100`: `approved: self.mode == Mode::Automated` — Auto mode auto-approves decisions
  - `dispatch.rs:107`: `self.mode == Mode::Safe && self.requires_decision_point(plan)` — Safe shows plan preview
  - `dispatch.rs:270-278`: `requires_decision_point()` checks high-level plans (team/workflow/jump_studio) and high-risk actions (delete/deploy/publish/payment)
  - Test `execution.rs:193-199`: Automated mode auto-approves single_tool but not workflow
- **BUT**: Safe mode doesn't actually block execution — it just logs a tracing message
- The behavioral differences are minimal and Safe mode does not enforce any real approval flow

### 3. COO 学习机制 (was ❌)
**Status: ✅ CLOSED**

- `src/core/supervisor/learning.rs` — **full LearningEngine** (370 lines, 15 tests):
  - `ingest_decision()` — records outcomes, creates/updates decision rules by keyword
  - `auto_adjust()` — downgrades unreliable rules when success rate < 60%
  - `handle_dont_ask_pattern()` — G23: learns "以后不用问我 / don't ask me" patterns → lowers threshold
  - `learn_from_correction_history()` — P5: batch correction learning
  - `trend_analysis()` — reports on frequently-used keywords
- Wired into `decide_level_with_context()` (decision.rs:122-130) and `execute_plan()` (dispatch.rs:209-211)

### 4. 对话修改规则 (was ❌)
**Status: ✅ CLOSED**

- `src/core/supervisor/rule_commands.rs` (152 lines, 7 tests):
  - `modify_rule_from_nl()` — NL command parsing
  - Supports: `add | <action> | <level> | <condition> | <effect>`, `delete <id>`, `list`, `find <action>`
  - Backed by `DecisionRuleStore` in storage
  - Full test coverage for all command types

---

## SECTION 8: Security Gaps

### 1. Dual-LLM 6 个检查点 (was 🟡)
**Status: ✅ CLOSED — All 6 checkpoints implemented**

All 6 checkpoints from `checkpoints.rs:5-12` are implemented in `engine.rs:run_checkpoint_mut()`:

| Checkpoint | Implementation | Status |
|---|---|---|
| `Auth` | Pattern match: api_key/password/secret/token | ✅ Regex-like |
| `ParamValidate` | Input length > 10000 chars → Flag | ✅ Works |
| `ContentSanitize` | 25 dangerous patterns (SQL, shell, prompt injection) | ✅ Robust |
| `Permission` | Delegates to `SecurityGuard::is_allowed()` | ✅ Real check |
| `Audit` | Appends to `AuditLog` | ✅ Real log |
| `Route` | Checks sandbox_level from SecurityProfile | ✅ Level-based |

- The checkpoint chain is **fully functional**
- The "Dual" part (two LLM judges) remains 🟡 — but the 6 checkpoints themselves are complete

---

## CODE QUALITY FINDINGS

### 1. Repeated CRUD Patterns
**Verdict: ACCEPTABLE — Entity-specific separation is appropriate**

- 8 storage entity modules each have entity-specific CRUD: agents (8), tasks (11), users (20), market (7), etc.
- Each operates on different SQLite tables with distinct schemas — not duplicative code
- Could use a generic CRUD trait, but current per-entity approach is idiomatic Rust

### 2. Module Doc (//!) Coverage
**Metric: 81.6% (125/153 files have module-level doc)**

- **Good coverage** for a project of this size
- 28 files missing module doc — mostly small/skeletal modules
- Missing: `orchestrator/chain.rs`, `orchestrator/broadcast.rs`, `agent_pool/selector.rs`, `pipeline/mod.rs`, `security/mod.rs`, `dual_llm/mod.rs`, `task_engine/mod.rs`

### 3. unwrap() in Non-Test Code
**Metric: 1 unwrap() in production code**

- All other unwrap() calls are inside `#[cfg(test)]` modules or dedicated test files
- **Single violation**: `src/core/model_router/local_engine.rs:24`
  ```rust
  let name = path.file_stem().unwrap().to_string_lossy().to_string();
  ```
  Will panic on paths without a stem (e.g., `.gguf` hidden files). Should use `unwrap_or_default()` or proper error handling.

### 4. Files >400 Lines (Split Candidates)
4 files exceed 400 lines:

| File | Lines | Issue |
|---|---|---|
| `src/core/storage/users/mod.rs` | 436 | Multi-entity CRUD in one file; split by entity |
| `src/core/workflow/mod.rs` | 415 | Workflow engine + templates + variable store mixed |
| `src/core/storage/mod.rs` | 407 | init_tables() is huge; schema DDL could be separate |
| `src/core/orchestrator/mod.rs` | 407 | Multiple collaboration responsibilities mixed |

### 5. Overly Fragmented Modules (<50 Lines, Merge Candidates)
23 files <50 lines. **Key merge candidates**:

| File | Lines | Suggestion |
|---|---|---|
| `orchestrator/chain.rs` | 22 | Merge into orchestrator/collaboration.rs |
| `orchestrator/broadcast.rs` | 17 | Merge into orchestrator/collaboration.rs |
| `agent_pool/selector.rs` | 26 | Merge into agent_pool/mod.rs |
| `agent_pool/stats.rs` | 48 | Merge into agent_pool/mod.rs |
| `orchestrator/voting.rs` | 44 | Merge into orchestrator/mod.rs |
| `orchestrator/blackboard.rs` | 32 | Merge into orchestrator/mod.rs |
| `pipeline/agentless.rs` | 41 | May need a dir; the file itself is short |
| `security/profile.rs` | 32 | Merge into security/guard.rs or security/mod.rs |
| `task_engine/mod.rs` | 6 | Just re-exports; could inline |

### 6. #[allow(dead_code)] Annotations
9 occurrences, 7 marked as "预留" (architecturally reserved):

| File | Line | Context |
|---|---|---|
| `core/worker.rs:10` | WorkerPool lifecycle | Reasonable |
| `core/assembler.rs:20,40` | Registry injection placeholder | Reasonable |
| `core/workflow_builder.rs:29` | Future validation | Reasonable |
| `core/agent_loop.rs:46` | Future event stream | Reasonable |
| `core/registry/capability.rs:2` | Market/routing stats | Reasonable |
| `core/orchestrator/mod.rs:91` | Runtime not fully wired | Reasonable |
| `core/pipeline/executor.rs:103` | DAG downstream traversal | Reasonable |
| `core/dual_llm/engine.rs:11` | Reserved fields | Reasonable |

**Verdict**: All dead_code annotations are architecturally justified — clean pattern for forward-looking code.

---

## SUMMARY

| Gap | Previous | Now | Evidence |
|---|---|---|---|
| **S2: Dual-LLM secondary** | 🟡 | 🟡 | `run_secondary_check()` exists but no production caller connects real LLM |
| **S2: Thread pool** | ❌ | 🟡 | ThreadPool struct exists with real threads but they're sleep-loops |
| **S2: Child process** | ❌ | ✅ | `child_process.rs` has full spawn/wait/kill with pipe I/O |
| **S2: Channel thread pool** | ❌ | ❌ | No channel→thread pool wiring anywhere |
| **S4: User override** | ❌ | ✅ | `override_decision()`, `DecisionOverride`, inline "L4:" syntax |
| **S4: 3 modes difference** | ❌ | 🟡 | Mode affects auto-approval + logging but Safe doesn't block |
| **S4: COO learning** | ❌ | ✅ | Full `LearningEngine` with 6 features + 15 tests |
| **S4: Rule commands** | ❌ | ✅ | `modify_rule_from_nl()` with add/delete/list/find |
| **S8: 6 checkpoints** | 🟡 | ✅ | All 6 implemented with real logic |

### Quality Summary
- **CRUD patterns**: Acceptable — entity-specific per file
- **Module docs**: 81.6% (good)
- **Production unwrap()**: 1 (minor risk in local_engine.rs)
- **>400 line files**: 4 (split recommended)
- **<50 line files**: 23 (several merge candidates listed)
- **dead_code**: 9 (all architecturally justified)
