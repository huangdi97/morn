# Morn — 轮3：进化系统补齐

## 约束
1. 不碰无关文件
2. 低耦合，模块间传ID不传对象
3. 文件不超300行
4. 所有已有测试保持通过

---

## 模块3A：L1 思维风格进化

### 新文件
- `morn_core/evolution/thinking_styles.py`

### 规格
ThinkingStyleEvolver 类：

**核心概念**：实例的"思维模板库"——一组经过提炼的推理路径，在遇到相似问题时优先检索和匹配。不是修改system prompt，不是修改对话模板，是修改"思考方式"的缓存。

**三步循环**：
1. **修订（Revise）** — 发现当前思维模板导致不佳结果时，标记该模板并生成修订版
2. **重组（Recombine）** — 将两个匹配度高的模板交叉重组，生成新模板
3. **提炼（Refine）** — 同一模板连续成功≥3次，提升其优先级

**实现**：
- 模板存储为结构化条目：`{template_id, name, trigger_conditions, reasoning_steps, success_count, fail_count, priority, created_at, last_used}`
- `register_thought(pattern, steps)` — 注册新思维模板
- `get_matching(context)` — 根据当前上下文匹配合适的模板
- `record_outcome(template_id, success)` — 记录模板使用结果
- `evolve()` — 执行一轮进化（修订/重组/提炼）
- 默认禁用（可通过配置开启）
- 存储路径：`{data_dir}/evolution/thinking_styles.json`

### 新增测试
`tests/test_thinking_styles.py` — 12+测试
- 注册匹配
- 结果记录
- 修订
- 重组
- 提炼
- 禁用状态

---

## 模块3B：L2.5 Harness 自优化

### 新文件
- `morn_core/evolution/harness.py`

### 规格
HarnessOptimizer 类：

**三大可观测性支柱驱动：**
1. **提示词质量** — 监控对话成功/失败比例，记录提示词有效性
2. **工具选择效率** — 记录每次工具调用耗时和成功率
3. **记忆参数效果** — 记录检索结果的用户采纳率（用户是否基于检索结果继续对话）

**自动闭环：**
- `collect_metrics()` — 从各子系统收集可观测性数据
- `diagnose()` — 识别瓶颈（如提示词命中率下降、工具超时率上升）
- `optimize(target)` — 生成优化建议（不含代码修改，仅参数调整建议）
- 所有优化记录到 `{data_dir}/evolution/harness_log.json`
- 默认禁用（可通过配置开启）

### 新增测试
`tests/test_harness.py` — 10+测试
- 指标收集
- 诊断
- 优化建议生成
- 日志记录
- 禁用状态

---

## 模块3C：进化日志

### 修改文件
- `morn_core/evolution/__init__.py` — 导出新增类

### 新文件
- `morn_core/evolution/evolution_log.py`

### 规格
EvolutionLogger 类：
- `log(source, action, detail)` — 记录进化事件（时间戳+来源+操作+详情）
- `get_log(limit=50, source=None)` — 查询进化日志
- `get_stats()` — 按来源统计进化次数
- 存储方式：只追加写入 `{data_dir}/evolution/evolution_log.jsonl`（每行一个JSON事件）
- 每次L3进化后运行人格基线测试，漂移超阈值时告警
- 集成：在L0 tuner、L2 skill manager、L1 thinking style evolver的进化触发点自动调用

### 新增测试
`tests/test_evolution_log.py` — 8+测试
- 事件记录
- 按来源筛选
- 统计
- 日志文件追加
- 文件不存在时自动创建

---

## 验收
1. 3个新模块的测试全部通过
2. `python3 -m pytest tests/ -q` — 全部通过
3. 进化日志自动集成到L0/L2/L1
4. 默认禁用状态的模块不干扰正常运行
