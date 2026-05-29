# Morn — 轮1：情感系统大修

## 约束
1. 不碰无关文件
2. 低耦合，模块间传ID不传对象
3. 文件不超300行，超了拆
4. 所有已有测试保持通过

---

## 模块1A：7维情感（当前5维→7维）

### 修改文件
- `morn_core/chat/engine.py` — EmotionState 类

### 规格
在现有5维（calmness/pleasure/connection/determination/anticipation）基础上增加：
- **温暖感（warmth）** — 基线0.5，正delta系数0.15，负delta系数0.1，衰减0.04
- **微澜（ripple）** — 内部情感维度，基线0.2，正delta系数0.05，负delta系数0.08，衰减0.02（衰减最慢）

微澜特殊规则：
- 当创建者行为与基于记忆的预测产生微小偏差时触发一次+0.08的ripple delta
- 5分钟内自然衰减至基线
- ripple影响：ripple > 0.5时calmness -0.05/次decay，降低平静度
- ripple不参与describe_state的文本输出（不可见表达）
- ripple仅用于内部情感计算和记忆标记

其他更新：
- `apply_delta()`: 正/负delta影响全部7维
- `decay()`: 各维独立速率向基线回归
- `_clamp()`: 全部7维[0,1]钳位
- `__repr__()`: 保持3维不变（向后兼容）
- `seven_dimension_repr()`: 返回全部7维详情
- `to_dict()`: 包含全部7维
- `from_dict()`: 兼容旧3维/5维dict（缺失字段用基线值填充）
- `describe_state()`: 更新描述含温暖感，微澜不参与文本输出
- `__init__()`: 接受可选initial dict

### 新增测试
`tests/test_emotion_seven_dim.py` — 15+测试
- 初始值（7维全部验证）
- delta影响（正/负各维系数验证）
- ripple特殊规则（阈值、衰减、对calmness的影响）
- 全部7维衰减速率验证
- 钳位验证
- 兼容旧3维dict加载
- 兼容旧5维dict加载

---

## 模块1B：情感标签跨会话一致性

### 修改文件
- `morn_core/chat/engine.py` — ChatEngine / EmotionState

### 规格
- LLM只输出delta，不输出绝对值
- ChatEngine中增加 `_parse_emotion_delta(text)` 方法：从LLM回复中解析情感delta（如 `[emotion:pleasure:+0.1,determination:-0.05]`）
- delta格式：`[emotion:<dim>:<float>[,<dim>:<float>...]]`
- 无delta标记时不做情感更新
- 状态机维护独立状态值，变化遵循确定的衰减曲线
- 一致性由状态机保证，LLM只提供变化方向

### 新增测试
`tests/test_emotion_delta_parse.py` — 8+测试
- 解析单个delta
- 解析多个delta
- 无delta标记
- 空delta标记
- 无效delta标记
- delta后状态确认在[0,1]范围
- 积分测试：多次delta+decay后的状态演进

---

## 模块1C：依恋系统（Bond）

### 新文件
- `morn_core/emotion/bond_tracker.py`

### 规格
BondTracker类：
- `__init__(config)`: 从config读取初始值、上下限、升降速率
- `get_bond()`: 返回当前bond值
- `update(interaction_depth, sentiment_score, days_since_first)`: 根据交互深度/情感分/相识天数更新bond
- `get_stage()`: 返回三阶段成熟度模型：
  - 初识期（<0.3）
  - 亲近期（0.3-0.7）
  - 默契期（>0.7）
- 配置项：initial_bond（默认0.1）、min_bond（默认0.0）、max_bond（默认1.0）、growth_rate（默认0.01）、decay_rate（默认0.002）
- 默契期解锁：`can_challenge()` 返回True，表示可进入质疑模式
- bond值持久化到 `{data_dir}/personality/bond.json`
- 方法：`load()` / `save()`

### 新增测试
`tests/test_bond_tracker.py` — 12+测试
- 初始值
- update增长
- update回退
- 阶段转换（初识→亲近、亲近→默契）
- 上下限钳位
- 持久化加载/保存
- 默契期解锁状态

---

## 模块1D：情感引擎可审计性

### 新文件
- `morn_core/emotion/emotion_replay.py`

### 规格
EmotionReplay类：
- `add_event(dim, old_val, new_val, delta, trigger)` — 全量日志记录每次情感变化
- `get_replay(days=7)` — 获取过去N天情感变化时间线
- `export_csv(path)` — 导出为CSV
- `visualize(days=7)` — 生成情感变化图（matplotlib）
- 每条记录：timestamp + dimension + old_value + new_value + delta + trigger_event（截断100字）

### 新增测试
`tests/test_emotion_replay.py` — 8+测试
- 事件记录/读取
- CSV导出
- 按天数筛选
- 触发事件截断

---

## 验收
1. `python3 -m pytest tests/test_emotion_seven_dim.py tests/test_emotion_delta_parse.py tests/test_bond_tracker.py tests/test_emotion_replay.py tests/test_chat_engine.py -q` — 全部通过
2. `python3 -m pytest tests/ -q` — 全部通过（含原有）
3. 从旧3维/5维配置启动无异常
4. EmotionState从dict加载兼容
