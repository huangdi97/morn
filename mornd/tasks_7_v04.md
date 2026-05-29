# Morn — 7.txt v0.4 交付：技能策展者 + SKILL.md加载器

## 约束
1. 不碰无关文件
2. 低耦合
3. 所有已有测试保持通过

---

## 模块1：技能策展者

### 新文件
- `morn_core/evolution/skill_curator.py`

### 规格
SkillCurator 类：

**职责：** 监控技能使用频率和成功率，标记低效/废弃技能。

**方法：**
- `review_skills(skills_list)` — 遍历所有技能，返回评估结果
- `get_stats(skill_id)` — 返回单个技能的使用统计（使用次数、成功率、最后使用时间）
- `mark_deprecated(skill_id)` — 标记技能为"可废弃"
- `propose_archive()` — 返回建议存档的技能列表及理由
- `get_review_report()` — 返回完整评估报告

**评估规则：**
- 90天未使用 → 标记为低使用率（但有保留价值）
- 90天未使用 + 成功率<80% → 可废弃
- 成功率<60% → 可废弃（不论使用频率）
- 同一功能有3个以上类似技能 → 建议合并

**集成：**
- 与 SkillManager 协同工作（读 skills 数据）
- 结果写入进化日志
- 默认启用（可通过配置关闭）

### 新增测试
`tests/test_skill_curator.py` — 12+测试
- 正常技能不标记
- 90天未使用标记低使用
- 90天未使用+低成功率标记可废弃
- 低成功率标记可废弃
- 相似技能合并建议
- 报告生成
- 配置禁用

---

## 模块2：SKILL.md 加载器

### 新文件
- `morn_core/evolution/skill_loader.py`

### 规格
SkillLoader 类：

**职责：** 从外部 SKILL.md 文件加载技能，注册到 SkillStore。

**方法：**
- `load_from_dir(dir_path)` — 从目录加载所有 SKILL.md 文件
- `load_from_file(file_path)` — 从单个 SKILL.md 文件加载
- `list_available(dir_path)` — 列出目录中可加载的 SKILL.md 文件
- `validate(file_path)` — 验证 SKILL.md 格式是否正确

**SKILL.md 格式：**
```yaml
---
name: skill-name
description: 技能描述
trigger_keywords: [关键词1, 关键词2]
template: 回复模板
source: external
---
技能正文（markdown格式，含使用说明）
```

**解析规则：**
- 兼容标准 SKILL.md YAML frontmatter
- name 缺失 → 跳过并记录警告
- trigger_keywords 缺失 → 使用文件名作为关键词
- 重复加载同名技能 → 跳过（幂等）
- 非法 frontmatter → 跳过并记录错误

**集成：**
- 注册到 SkillStore（现有 `store.py` + `manager.py`）
- 支持双轨制（外部加载 + 内部生长）
- 加载结果写入进化日志

### 新增测试
`tests/test_skill_loader.py` — 12+测试
- 加载有效 SKILL.md
- 加载无效格式跳过
- 加载目录
- 列出可用
- 幂等（重复不加载）
- name缺失跳过
- keyword缺失用文件名
- frontmatter解析错误

---

## 验收
1. `python3 -m pytest tests/ -q` — 全部通过
2. 策展者默认启用
3. SKILL.md 格式解析正确
4. 双轨制统一管理
