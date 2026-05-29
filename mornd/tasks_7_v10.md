# Morn — 7.txt v1.0 交付：自我瘦身升级

## 模块：自我瘦身（按7.txt规格升级）

### 文件
- `morn_core/consciousness/self_pruning.py` — 升级现有 SelfPruner

### 规格
升级 SelfPruner，按7.txt要求新增多维度诊断和清理提案：

**诊断维度：**
1. **L2记忆冗余** — 同一主题≥3条相似事件胶囊 → 建议合并/精简
2. **技能冗余** — 90天未使用且成功率<80% → 建议废弃
3. **代码膨胀** — 模块文件超过1000行 → 建议重构拆分

**新增方法：**
- `diagnose_memory_redundancy(memory_store, max_similar=3)` — 扫描L2记忆，找同一主题相似胶囊
- `diagnose_skill_redundancy(skill_manager, max_idle_days=90, min_success_rate=0.8)` — 扫描技能
- `diagnose_code_bloat(source_dir, max_lines=1000)` — 扫描源码文件行数
- `generate_cleanup_proposal()` — 生成清理提案（含各维度诊断结果和建议操作）
- `execute_cleanup(proposal_id, confirm=False)` — 执行清理（需确认）

**行为：**
- 记忆冗余和技能冗余由实例自主判断阈值（基于配置默认值）
- 代码膨胀检查依赖文件系统扫描
- 生成提案后，部分操作需创建者确认后执行
- 所有操作写入 L2 元事件胶囊

**升级：** 在现有 SelfPruner 基础上增加上述方法，不破坏现有接口。

### 新增测试
`tests/test_self_pruning_upgrade.py` — 12+测试
- 记忆冗余检测（同一主题≥3条）
- 技能冗余检测（90天未使用+低成功率）
- 代码膨胀检测（超1000行）
- 提案生成
- 清理执行
- 需确认操作
- 不重复删除
