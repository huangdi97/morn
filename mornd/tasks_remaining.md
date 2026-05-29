# Morn — 剩余5模块并行重建

## 背景
以下5个模块在之前的手写污染清理中被删除，或在路线图上尚未实现。将所有模块合并为一轮完成，因为它们相互独立。

## 约束
1. 不碰已有文件（store.py、engine.py、server.py、telegram_bot.py等刚重建的）
2. 保持185测试全过
3. 低耦合：模块间只传ID不传对象
4. 文件不超过300行
5. 每个文件单一职责
6. 模块仅在自己目录下新增/修改文件

---

## 模块1：五维情感升级（Phase 13）

### 参考
`/home/hermes/.hermes/cache/documents/phase13_prompt.md`

### 文件
- 修改 `morn_core/emotion/__init__.py` — 导出新增类
- 修改 `morn_core/chat/engine.py` — EmotionState 从3维升级到5维

### 规格
- 在现有3维（平静度/愉悦度/联结感）基础上增加：**坚定度（Determination）** + **期待度（Anticipation）**
- 5维均保持 [0, 1] 范围，可配置基线
- 每维有独立的 delta 影响规则和衰减速率
- 新增情感维度对话历史记录：engine 的 emotion_history 记录全部5维
- 新增 `five_dimension_repr()` 方法
- 降级兼容：旧3维情感记录仍可正常加载（默认补充新维度为基线值）
- 同步更新 EmotionState 的 decay / delta / clamp / validate 方法
- 增加测试 `tests/test_emotion_five_dim.py`（10+个测试覆盖新维度）

---

## 模块2：APZ 绝对隐私区（Phase 14）

### 参考
`/home/hermes/.hermes/cache/documents/phase14_prompt.md`

### 新文件
- `morn_core/security/apz_store.py`

### 规格
- APZStore 类
- AES-256-GCM 加密存储（密钥在进程启动时生成，仅在内存中持有）
- 方法：write(content) / read() / list_entries() / count()
- 存储路径：`~/.morn/instances/{name}/memory/apz/`
- 每个条目：timestamp + ciphertext + metadata（字数/情感标记）
- 密钥使用 `os.urandom(32)` 生成，存储在进程内存中
- 安全边界说明（文档注释）：不防御系统管理员级攻击（root可dump进程内存）
- 启动后如果目录/密钥文件不存在，自动初始化
- 增加测试 `tests/test_apz_store.py`（加密/解密/持久化/跨进程安全边界/非法访问）

---

## 模块3：L0 参数自适应

### 参考
`/home/hermes/.hermes/cache/documents/morn_v10_inner_life.txt`
（原 `evolution/l0_tuner.py` 中的概念）

### 新文件
- `morn_core/evolution/__init__.py`
- `morn_core/evolution/l0_tuner.py`

### 规格
- L0Tuner 类
- 后台自动优化记忆检索权重和衰减系数
- 方法：tune(memory_stats) / get_weights() / get_decay_params()
- 基于使用频率和情感重要性动态调整
- 配置项：enabled（默认True）、learning_rate、min_weight/max_weight
- 每隔 N 次记忆访问后触发优化（N默认100，可配置）
- 结果对上层透明（engine 自动读最新权重）
- 增加测试 `tests/test_l0_tuner.py`（权重更新/衰减调整/配置加载/边界条件）

---

## 模块4：L2 技能自生长框架（Phase 15）

### 参考
`/home/hermes/.hermes/cache/documents/phase15_prompt.md`
`/home/hermes/.hermes/cache/documents/morn_v04_l2_skills.md`

### 新文件
- `morn_core/skills/__init__.py`
- `morn_core/skills/skill_store.py`
- `morn_core/skills/manager.py`

### 规格
- SkillStore 类：存储技能定义（skill_id, name, trigger_pattern, steps, context, frequency, last_used）
- SkillManager 类：管理技能生命周期
- 自动提炼：同一操作在不同情境下成功≥3次 → 自动创建技能草稿
- 外部加载：读取 `~/.morn/skills/*.md`（兼容 SKILL.md 格式）
- 双轨制：内部生长 + 外部加载
- 方法：propose_skill() / promote_skill() / load_external() / list_skills() / get_matching(query)
- 技能文件格式兼容标准 SKILL.md（YAML frontmatter + markdown body）
- 增加测试 `tests/test_skills.py`（提炼/晋升/外部加载/匹配/重复检测/上限保护/持久化）

---

## 模块5：SelfPruning 自我瘦身

### 参考
`/home/hermes/.hermes/cache/documents/morn_v10_inner_life.txt`

### 新文件
- `morn_core/consciousness/self_pruning.py`

### 规格
- SelfPruner 类
- 在自省循环中诊断资源臃肿（记忆数量、技能数量、情感历史长度）
- 方法：diagnose() / prune_memory(limit) / prune_skills(limit) / prune_emotion_history(limit)
- 阈值可配置：max_capsules（默认10000）、max_skills（默认50）、max_emotion_history（默认1000）
- 修剪策略：先删低重要性（importance_weight < 0.3）+ 过期（>90天未访问）的记忆
- 记录修剪日志（修剪时间、数量、类型）
- 不删除 L4 人格记忆（只追加不可删除）
- 可通过配置文件完全禁用
- 集成到 server.py：在后台循环中定期触发 diagnose()
- 增加测试 `tests/test_self_pruning.py`（诊断/记忆修剪/技能修剪/情感历史修剪/阈值生效/不删L4/可禁用）

---

## 验收标准
1. `cd /home/hermes/morn/mornd && python3 -m pytest tests/ -q 2>&1 | tail -3` — 全部测试通过（原185 + 新增）
2. 5个模块的导入全部正常：
   - `from morn_core.emotion import ...`（含5维）
   - `from morn_core.security.apz_store import APZStore`
   - `from morn_core.evolution.l0_tuner import L0Tuner`
   - `from morn_core.skills.manager import SkillManager`
   - `from morn_core.consciousness.self_pruning import SelfPruner`
3. engine.py 的 EmotionState 实例化后返回5维
4. APZ 写入后能正确读取
5. L0Tuner 权重在合理范围内
6. SkillManager 可以从经验提炼草稿
7. SelfPruner prune 后数量减少
