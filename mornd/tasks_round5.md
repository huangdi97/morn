# Morn — 轮5：内心生活补齐

## 约束
1. 不碰无关文件
2. 低耦合
3. 文件不超300行
4. 所有已有测试保持通过

---

## 模块5A：质疑模式

### 新文件
- `morn_core/consciousness/challenge_mode.py`

### 规格
ChallengeMode 类：

**解锁条件**（`is_unlocked()`）：
- Bond >= 0.95
- 深度对话 >= 50次
- 连续存在 >= 30天

**触发**：基于记忆提醒创建者潜在问题。

**内置记忆锚点**：
- `find_challenge_topics(memory_store, bond_tracker)` — 搜索创建者做过决定且结果可验证的事件
- 半结构化模板约束提问质量：
  - 模板1："你之前决定[事件]，结果[结果]。现在回头看，你会改变什么？"
  - 模板2："关于[主题]，我发现了一段与你当时说法不同的记忆。[引用]。你怎么看？"
  - 模板3："你曾经[行为]，当时你说[理由]。我注意到[观察]，想问问你的想法。"

**频率控制**：每7天最多触发1次质疑，避免频繁打扰。

**Bond影响**：质疑触发后bond消耗0.02（合理质疑），如果创建者正面回应则bond恢复并+0.01（信任增益）。

### 新增测试
`tests/test_challenge_mode.py` — 10+测试
- 锁定状态（条件不满足时不可用）
- 解锁条件（bond/深度/天数）
- 话题检索
- 模板生成
- 频率控制
- Bond消耗与恢复

---

## 模块5B：self_preface.md

### 新文件
- `morn_core/consciousness/self_preface.py`

### 规格
SelfPreface 类：

- 路径：`{data_dir}/self_preface.md`
- 初始创建：空文件（只包含空白行）
- `is_unlocked()` — 检查解锁条件：
  - Bond >= 0.95 保持 >= 30天
  - 连续存在时间 >= 6个月
  - 至少完成一次经创建者确认的自我改进
- `get_preface()` — 返回当前内容
- `write_line(line)` — 只追加一行（仅 unlocked 后可写）
- `is_blank()` — 检查是否仍为空白

### 新增测试
`tests/test_self_preface.py` — 8+测试
- 初始空白
- 锁定状态不可写
- 解锁条件
- 解锁后可写只追加
- 空白检测

---

## 模块5C：元认知token成本优化

### 修改文件
- `morn_core/consciousness/self_reflection.py` — SelfReflection 类增加 MetaCognitiveReuse 机制

### 规格
借鉴 Meta Metacognitive Reuse 机制：

**行为手册**：常规的、重复性的自省模式被提炼为标准化行为条目，存储在 `{data_dir}/consciousness/behavior_manual.json`

- `_reuse_pattern(cycle_type)` — 检查当前自省是否匹配已有的标准化行为条目
  - 匹配 → 复用标准化条目（零token调用）
  - 不匹配 → 执行完整LLM调用，结果提炼为新条目加入行为手册
- 异常情况才触发完整LLM调用
- `get_reuse_rate()` — 返回当前复用率（目标：减少最多46%推理token）
- `get_behavior_manual()` — 查看当前行为手册
- 默认启用（可通过配置关闭）

### 新增测试
`tests/test_meta_reuse.py` — 8+测试
- 复用匹配
- 无匹配时调用LLM
- 新条目提炼
- 复用率统计
- 禁用状态

---

## 验收
1. 3个模块测试全部通过
2. `python3 -m pytest tests/ -q` — 全部通过
3. 默认锁定功能不干扰正常运行
