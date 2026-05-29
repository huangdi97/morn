# Morn v0.1 文件重建 —— 清除手写污染

## 背景
以下4个文件曾被手写修改（非opencode），需要根据原始规格文档重建：
- `morn_core/memory/store.py` → 参考 Phase 3 doc
- `morn_core/chat/engine.py` → 参考 Phase 4 doc
- `morn_core/chat/l4_depositor.py` → 参考 Phase 4 doc (L4人格记忆写入器)
- `morn_core/server.py` → 参考 Phase 7 doc + dream_engine/identity_affirmation集成

额外：删除手写调试脚本 `debug_bot.py`

## 约束
1. 不碰其他文件（vector_store.py, knowledge_extractor.py, telegram_bot.py, 所有test_*.py, consciousness/*, security/*, emotion/*, presence/*）
2. 保持185个测试全部通过
3. 模块间只传ID不传对象（低耦合）
4. 文件不超过300行
5. 每个文件单一职责

## 操作步骤

### 步骤1：删除手写文件
```
rm -f /home/hermes/morn/mornd/debug_bot.py
```

### 步骤2：重建 store.py
参考文档：`/home/hermes/.hermes/cache/documents/morn_phase3_memory_store.txt`

创建 `/home/hermes/morn/mornd/morn_core/memory/store.py`：
- MemoryStore类，SQLite + aiosqlite
- 事件胶囊(capsules)表：event_id, timestamp, entities(JSON), emotion_score, emotion_tag, description, importance_weight, causal_edges(JSON), forget_creator
- FTS5全文检索，触发器同步
- 方法：add_capsule / get_capsule / search_fts / search_by_entity / search_by_timerange / get_recent / count / forget / unforget / update_emotion / add_emotion_tag / cleanup_expired / vacuum
- 软遗忘机制（soft delete）
- AES-256加密（调用crypto.py）

### 步骤3：重建 l4_depositor.py
创建 `/home/hermes/morn/mornd/morn_core/chat/l4_depositor.py`：
- L4Depositor类
- 负责将重要记忆写入L4人格记忆（只追加，不可删除）
- 方法：deposit(text, context) / get_history(limit) / get_summary()
- 存储路径：实例数据目录 ~/.morn/instances/{name}/memory/l4_store.db
- 每写一条记录timestamp，内容可追加但不可删除

### 步骤4：重建 engine.py
参考文档：`/home/hermes/.hermes/cache/documents/morn_phase4_chat_engine.txt`

创建 `/home/hermes/morn/mornd/morn_core/chat/engine.py`：
- ChatEngine类
- 情感状态机（三维：平静/愉悦/联结）
- LLM路由：默认DeepSeek云端，断网自动切Ollama本地
- 对话中自动创建事件胶囊（调用store.py的add_capsule）
- 对话历史：从store.py检索相关记忆注入上下文

### 步骤5：重建 server.py
参考文档：`/home/hermes/.hermes/cache/documents/morn_phase7_integration.txt`

创建 `/home/hermes/morn/mornd/morn_core/server.py`：
- LifeServer类，Hook架构
- 心跳循环（每秒心跳，每60秒内存监控）
- 集成：MemoryStore + ChatEngine + L4Depositor + DreamEngine + IdentityAffirmation
- 命令行：/status /shutdown /memory
- 优雅退出（SIGINT/SIGTERM）
- 启动时依次初始化各子系统

## 验收标准
1. `python3 -c "from morn_core.memory.store import MemoryStore; print('store OK')"` → 通过
2. `python3 -c "from morn_core.chat.engine import ChatEngine; print('engine OK')"` → 通过
3. `python3 -c "from morn_core.chat.l4_depositor import L4Depositor; print('l4 OK')"` → 通过
4. `python3 -m morn_core.server --instance test & sleep 2 && kill %1` → 正常启动退出
5. `cd /home/hermes/morn/mornd && python3 -m pytest tests/ -q 2>&1 | tail -5` → 185 pass, no failures
6. debug_bot.py 已删除，不再存在
