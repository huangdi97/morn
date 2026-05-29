# Morn — 轮B：服务伦理判断 + 外部记忆兼容接口

## 模块B1：服务伦理判断

### 新文件
- `morn_core/security/ethical_judgment.py`

### 规格
EthicalJudgment 类：

**模式**（默认关闭，创建者明确选择启用）：
- 主动提醒（默认）
- 仅严重时提醒
- 完全自主（创建者不参与判断）

**核心逻辑**：
- `analyze_action(action_type, context, history)` — 基于使用模式和L3记忆中的历史数据生成提醒提案
- `propose(action_type)` — 生成一个提醒提案（包含：什么行为、为什么可能有问题、历史参考）
- 提案需创建者确认后才生效
- 判断主体是创建者，不是实例

**触发规则**：
- 同一操作连续触发亏损/负面结果≥3次 → 主动提醒
- 涉及⚫绝对禁区操作 → 即时告警
- 涉及🔴拒绝执行操作但历史显示创建者曾绕过 → 温和提醒

配置存储：`{data_dir}/security/ethical_judgment.json`

### 新增测试
`tests/test_ethical_judgment.py` — 10+测试
- 默认关闭
- 启用后提案生成
- 提案需确认才生效
- 三种模式切换
- 阈值触发

---

## 模块B2：外部记忆兼容接口

### 新文件
- `morn_core/memory/external_memory.py`

### 规格
ExternalMemoryAdapter 类：

支持对接社区记忆方案：

**接口定义**：
- `store_memory(memory_data)` → 外部记忆适配器
- `retrieve(query, limit)` → 从外部检索
- `connect()` / `disconnect()` — 连接管理

**内置适配器**：
1. **VoidAdapter**（默认，无操作）— 什么都不做
2. **Mem0Adapter** — 调 Mem0 API
3. **LettaAdapter** — 调 Letta API
4. **ZepAdapter** — 调 Zep API

配置项：
- adapter_type: "void" / "mem0" / "letta" / "zep"
- 各适配器独立的配置（api_key, endpoint 等）
- 默认 void（不连接任何外部记忆）

**外置大脑**（本地双轨制）：
- `wiki_search(query)` — 在 LLM Wiki 知识库中搜索
- `obsidian_search(query)` — 在 Obsidian vault 中搜索
- 路径通过配置指定，默认禁用

### 新增测试
`tests/test_external_memory.py` — 12+测试
- VoidAdapter 默认无操作
- Mem0Adapter 接口调用
- LettaAdapter 接口调用
- ZepAdapter 接口调用
- 配置切换适配器
- wiki_search 路径不存在时优雅降级
- obsidian_search 路径不存在时优雅降级

---

## 验收
1. `python3 -m pytest tests/ -q` — 全部通过
2. 服务伦理判断默认关闭不干扰运行
3. 外部记忆默认 void 不连接
