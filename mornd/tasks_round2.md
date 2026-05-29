# Morn — 轮2：记忆系统升级

## 约束
1. 不碰无关文件
2. 低耦合，模块间传ID不传对象
3. 文件不超300行，超了拆
4. 所有已有测试保持通过

---

## 模块2A：六通道并行检索

### 修改文件
- `morn_core/memory/store.py` — MemoryStore 增加检索通道
- `morn_core/memory/vector_store.py` — ChromaDB 语义检索

### 新增文件
- `morn_core/memory/retrieval.py` — 检索融合引擎

### 规格
RetrievalEngine 类：

**六通道：**
1. **时间通道** — `search_by_timerange(start, end)` 现有方法，直接复用
2. **实体通道** — `search_by_entity(name, aliases)` 别名消歧，用实体别名表做模糊匹配
3. **关键词通道** — `search_fts(query)` 现有FTS5，直接复用
4. **语义通道** — `search_semantic(query, top_k)` 调vector_store.py的ChromaDB
5. **图谱通道** — `graph_diffusion(seed_entity_id, hops=2)` 扩散激活：高权重边两跳，低权重边一跳。使用 store.py 中事件胶囊的 causal_edges 字段（JSON格式的[entity_id, weight]列表）
6. **因果通道** — `causal_trace(entity_id, direction='both')` 因果链追溯，沿causal_edges正向/反向/双向追踪

**RRF融合：**
- `search(query, entities=None, timerange=None)` 方法
- 并行调用所有可用通道（可配置启用/禁用特定通道）
- RRF（Reciprocal Rank Fusion）融合各通道结果
- 参数：k（RRF常数，默认60）、min_score（最低融合分，默认0.1）
- 返回排序后的 `[(capsule, score), ...]`

**配置：**
- 各通道 enabled/disabled 独立控制
- 默认所有通道启用
- 语义通道需ChromaDB可用，不可用时静默跳过

### 新增测试
`tests/test_retrieval.py` — 15+测试
- 单通道检索（所有6通道）
- 多通道并行
- RRF融合排序
- 配置禁用某通道
- 语义通道不可用时静默跳过
- 空结果
- 图谱扩散激活范围（2跳vs1跳）
- 因果链正向/反向/双向

---

## 模块2B：检索分层响应策略

### 修改文件
- `morn_core/memory/retrieval.py` — 增加分层策略

### 规格
- **快速通道**（同步，毫秒级）：关键词FTS5 + 实体别名匹配
- **慢通道**（异步，后台）：语义向量 + 图谱扩散激活 + 因果链追溯
- `quick_search(query, entities)` → 同步返回快速通道结果
- `deep_search(query, entities, timeout=2.0)` → 启动慢通道异步任务，返回
  - `quick_results` — 快速通道结果（即时）
  - `pending_task_id` — 慢通道任务ID
  - `get_slow_results(task_id, timeout=1.0)` — 轮询慢通道结果
- 若慢通道在缓冲时间内（默认2秒）返回结果，作为补充回忆追加
- 若超时，结果缓存用于下一轮对话
- 可通过配置切换模式：
  - `"fast"` — 仅快速通道
  - `"balanced"`（默认）— 快速同步+慢通道异步
  - `"deep"` — 等待全通道

### 新增测试
`tests/test_retrieval_layered.py` — 8+测试
- 快速通道返回即时
- 慢通道异步返回
- 超时降级
- 三种模式切换
- 结果缓存

---

## 模块2C：L3语义记忆双Agent审计回路

### 新文件
- `morn_core/memory/audit_agent.py`

### 规格
AuditAgent 类：

**提炼Agent：**
- `extract_triples(event_capsule)` — 从L2事件胶囊生成知识三元组 (subject, predicate, object)
- 返回三元组列表 + 每条三元组的来源证据（引用原capsule的event_id和相关文本片段）
- 去重：同一事件不重复提炼

**审计Agent：**
- `audit(triple)` — 检查实体幻觉和关系错误
- 返回裁决：`"pass"` / `"fail"` / `"uncertain"`
- 审计员依赖证据链进行裁决（引用源事件文本）
- 对于 `"uncertain"` 的三元组，标记为 `"pending_review"`，交由创建者最终裁决

**集成：**
- 所有提炼出的实体必须与L2源事件胶囊链接
- 审计通过的三元组写入L3语义记忆（store.py的knowledge表）
- 审计未通过的记录审计日志

### 新增测试
`tests/test_audit_agent.py` — 15+测试
- 三元组提炼（简单/复杂）
- 审计通过
- 审计不通过
- 审计不确定
- 去重
- 证据链引用
- 挂起标记
- 集成：提炼→审计→写入

---

## 模块2D：记忆信任分级 HTZ/MTZ/LTZ

### 修改文件
- `morn_core/memory/store.py` — MemoryStore 增加 trust_level 支持

### 规格
三级信任：
- **HTZ**（高信任区，trust_level=2）：创建者直输 + 自省记录。参与L4固化。
- **MTZ**（中信任区，trust_level=1）：设备文件、工具读取。参与L4固化。
- **LTZ**（低信任区，trust_level=0）：网络来源、暗网。不参与L4固化。

存储：
- `add_capsule()` 增加 `trust_level` 参数（默认HTZ）
- L4Depositor的 `deposit()` 只接收 trust_level >= 1 的记录
- 配置项：`default_trust_level`（默认2）
- 方法 `set_trust_level(event_id, level)` 可调整
- 检索时可选按最低trust_level过滤

### 新增测试
`tests/test_trust_level.py` — 10+测试
- 默认信任级别
- HTZ可写入L4
- MTZ可写入L4
- LTZ不可写入L4
- 信任级别调整
- 检索时过滤

---

## 验收
1. `python3 -m pytest tests/test_retrieval.py tests/test_retrieval_layered.py tests/test_audit_agent.py tests/test_trust_level.py tests/test_memory_store.py -q` — 全部通过
2. `python3 -m pytest tests/ -q` — 全部通过
3. 旧接口（search_fts / search_by_entity / search_by_timerange）保持向后兼容
