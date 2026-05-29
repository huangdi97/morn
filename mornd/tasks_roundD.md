# Morn — 轮D：Redis + Neo4j 基础设施增强

## 模块D1：Redis L1 工作记忆缓存

### 新文件
- `morn_core/memory/redis_cache.py`

### 规格
RedisCache 类：

**用途**：高频快照和L1工作记忆缓存，替代内存列表。

`connect(host='localhost', port=6379, db=0)` — 连接Redis
`disconnect()` — 断开连接
`is_connected()` — 检查连接状态
`set(key, value, ttl=None)` — 写入缓存（可选TTL秒）
`get(key)` — 读取缓存
`delete(key)` — 删除

**使用场景**：
- 情感状态快照缓存（每60秒）
- 当前会话上下文缓存
- 临时任务状态

**降级**：Redis不可用时静默退回到内存模式，不影响主功能。
**配置**：`{redis_enabled: false, redis_host, redis_port, redis_db}` — 默认不启用。

### 新增测试
`tests/test_redis_cache.py` — 8+测试
- 连接/断开
- 读写
- TTL过期
- Redis不可用时静默降级

---

## 模块D2：Neo4j 知识图谱（自建轻量版）

### 新文件
- `morn_core/memory/graph_store.py`

### 规格
GraphStore 类：

**用途**：实体关系遍历 + 扩散激活，替代Neo4j的轻量自建图谱。

**核心结构**：
- 节点（entity_id, entity_type, name, aliases, metadata）
- 边（source_id, target_id, relation_type, weight, confidence）

**方法**：
- `add_node(entity_id, entity_type, name, aliases=None, metadata=None)`
- `add_edge(source_id, target_id, relation_type, weight=1.0, confidence=1.0)`
- `get_node(entity_id)` — 获取节点
- `get_edges(entity_id, direction='both')` — 获取边的连接（输入/输出/双向）
- `diffusion(entity_id, hops=2, min_weight=0.3)` — 扩散激活：从种子节点沿边扩散，高权重边两跳，低权重边一跳
- `search(query)` — 按名称/别名搜索节点
- `count_nodes()` / `count_edges()` — 统计

**存储**：两个CSV文件 `{data_dir}/graph/nodes.csv` 和 `{data_dir}/graph/edges.csv`

### 新增测试
`tests/test_graph_store.py` — 10+测试
- 增删节点
- 增删边
- 扩散激活（1跳/2跳）
- 权重过滤
- 搜索别名
- 持久化（CSV读写）

---

## 验收
1. `python3 -m pytest tests/ -q` — 全部通过
2. Redis默认不启用，不影响现有功能
3. GraphStore CSV持久化正常工作
