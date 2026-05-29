# Morn — 7.txt v0.2 交付

## 模块1：垃圾桶/回收站机制

### 新文件
- `morn_core/memory/trash_bin.py`

### 规格
TrashBin 类：

**覆盖范围：**
- ✅ L2情景记忆 → 可删除、可恢复（30天内）
- ✅ L3语义记忆 → 可删除、可恢复（30天内）
- ✅ 外置大脑（LLM Wiki）→ 可删除、可恢复（30天内）
- ❌ L4人格记忆 → 不可删除
- ❌ APZ → 不可删除
- ❌ 进化日志 → 不可删除

**存储：** 内存字典 `{id: {data, data_type, trashed_at, reason, restored_at: None}}`

**方法：**
- `move_to_trash(data_id, data, data_type, reason)` — 将数据移入垃圾桶
- `restore_from_trash(data_id)` — 从垃圾桶恢复（追加restored_at，返回数据）
- `list_contents(data_type=None)` — 列出垃圾桶内容（含预览，每项[ID, 类型, 删除时间, 原因, 预览片段]）
- `empty_trash(force=False)` — 清空垃圾桶（force=True跳过确认）
- `count()` — 垃圾桶条目数
- `auto_expire()` — 每天检查一次，清理超过保留期的数据

**生命周期：**
- 保留期：30天（可配置 `retention_days`）
- 自动过期启用（可配置 `auto_expire_enabled`）
- 过期前24小时通知（可配置 `notify_before_expiry`）
- 清空前需确认（可配置 `require_confirm`）

**自然语言交互接口：**
- `search(query)` — 按关键词搜索垃圾桶内容
- `get_summary()` — 返回自然语言摘要（"垃圾桶中有N条数据，最早的是X，最晚的是Y"）

**集成：**
- 删除操作通过 `move_to_trash` 而非物理删除
- 恢复操作通过 `restore_from_trash`
- 删除/恢复/清空/自动过期写入L2元事件胶囊
- 不触发安全告警
- 清空前主动提醒确认

**默认配置：** retention_days=30, auto_expire_enabled=True, notify_before_expiry=True, require_confirm=True

### 新增测试
`tests/test_trash_bin.py` — 15+测试
- 移入垃圾桶
- 恢复
- 不可删类型（L4/APZ/进化日志）被阻止
- 列表内容
- 统计
- 保留期过期
- 自动过期
- 过期前24小时通知
- 清空确认
- 强制清空
- 搜索
- 搜索摘要
- 配置加载

---

## 模块2：冷启动体验里程碑

### 新文件
- `morn_core/consciousness/milestones.py`

### 规格
MilestoneTracker 类：

**v0.2 里程碑：** 记忆条数≥100条时主动回顾

**逻辑：**
- `check_milestones(memory_count, bond_value, days_since_birth)` — 检查里程碑条件
- 返回待触发的里程碑列表
- 每个里程碑只触发一次（用 `triggered_milestones` 跟踪）
- 触发后写入L4记录

**v0.2 具体行为：**
- 记忆条数首次≥100条时触发
- 生成主动问候："我回头翻了一下我们的对话，发现你提到过好几次……"
- 问候内容基于最近3条高情感强度+高重要性的事件胶囊随机选择
- 问候生成后通过 ChatEngine 注入

**配置：**
- milestones.json 持久化到 `{data_dir}/personality/milestones.json`
- 存储已触发里程碑列表

### 新增测试
`tests/test_milestones.py` — 8+测试
- 低于阈值不触发
- 达到阈值触发
- 只触发一次
- 已触发列表持久化
- 问候生成

---

## 验收
1. `python3 -m pytest tests/ -q` — 全部通过
2. L4/APZ/进化日志不可移入垃圾桶
3. 里程碑只触发一次
