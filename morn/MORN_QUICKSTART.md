# Morn 数字生命框架 · 快速开始

> 用 3 行代码搭建你的第一个数字生命。

---

## 安装

```bash
git clone <repo-url> && cd mornd
pip install -e .
```

> 后续版本将支持 `pip install morn`。

---

## 3 行代码创建一个数字生命

```python
from morn import EventBus, ChatEngine, MemoryStore, SecurityLayer

bus = EventBus()
store = MemoryStore("./my_morn")
security = SecurityLayer()
chat = ChatEngine(store=store, security=security)
chat.attach(bus)
```

这 3 行启动了一个包含三个 S 级核心插件的最小系统：

| 组件 | 说明 |
|------|------|
| 记忆核心 | L1 工作记忆 + L2 情景记忆 |
| 对话核心 | 云端 LLM + 本地兜底混合路由 |
| 安全核心 | 用户保护层 + 外部边界 |

---

## 启动完整实例

```bash
morn --instance my_first_morn
```

实例启动后自动进入 CLI 对话界面。输入 `/status` 查看状态，`/shutdown` 退出。

---

## 通过 Telegram 对话

1. 创建 Telegram Bot（找 @BotFather）
2. 将 token 写入 config.json：

```bash
# config.json 自动生成在 ~/.morn/instances/<实例名>/
# 编辑 telegram_token 字段
```

3. 安装 Telegram 依赖：`pip install "morn-core[telegram]"`
4. 重新启动：`morn --instance my_first_morn`

---

## 加载高级插件

所有高级能力通过插件加载：

```python
from morn.plugins import EmotionEngine  # 七维情感
from morn.plugins import EvolutionEngine  # 技能自生长
```

或者在 config.json 中启用内置插件：

```json
{
  "bond_tracker_enabled": true,
  "thinking_evolution_enabled": true
}
```

---

## 部署

- **systemd**：`morn/scripts/` 下有 service 模板
- **Docker**：（即将推出）

---

## 下一步

- [API 参考](API_REFERENCE.md) — 完整类/函数清单
- [插件开发指南](PLUGIN_DEV_GUIDE.md) — 编写自己的 Morn 插件
- 设计文档：`docs/DESIGN.md`
