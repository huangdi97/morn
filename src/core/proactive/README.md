# ProactiveEngine

模块路径: `src/core/proactive/`

## 功能

定时/事件驱动的主动 Agent 框架。让 Agent 在后台自动执行任务，无需用户主动触发。

## 核心类型

```rust
pub enum Trigger {
    Timer(u64),   // 每 N 个 tick 触发一次
    Event(String), // 当匹配的事件发生时触发
}

pub struct ProactiveAgent {
    pub agent_id: String,
    pub trigger: Trigger,
    pub action: String,
    pub enabled: bool,
}
```

## 使用

```rust
let mut engine = ProactiveEngine::new();
engine.register(ProactiveAgent {
    agent_id: "monitor-1".into(),
    trigger: Trigger::Timer(10),  // 每 10 tick
    action: "check_system_health".into(),
    enabled: true,
});

engine.tick();                    // 推进 timer
let ready = engine.check_ready(None);  // 获取就绪的 Agent
```

## Tick 机制

`tick()` 递增内部计数器。`check_ready(event)` 检查哪些 Agent 的触发条件满足：
- Timer: counter >= interval 时返回，counter 重置为 0
- Event: event 参数与 Trigger::Event 匹配时返回
