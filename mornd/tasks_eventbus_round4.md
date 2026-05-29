# Morn v0.4 第四轮：遗留循环迁移 + 双注册修复

## 编程原则

### 14条核心准则
1. Think Before Coding 2. Simplicity First 3. Surgical Changes 4. Goal-Driven Execution
5. 架构优先拒绝补丁 6. 面向组件的构建 7. 显式优于隐式 8. 代码整洁自文档化
9. 单一职责 10. 组合优于委托 11. 单一状态源 12. 避免语法糖
13. 命名一致性 14. 文件不超过300行

### 低耦合原则
- 模块通过事件ID通信，不直接耦合
- 所有修改通过 opencode run 执行

---

## 任务 1：修复 SelfReflection 双注册

**文件**：`morn_core/server.py`

**问题**：
- 第 388-393 行注册了 `self_reflection` 作为 Hook（事件驱动）
- 第 423 行又创建了 `self_reflection.reflection_loop()` 作为独立 task
- 同一个插件的逻辑跑了两次

**解决方案**：删除第 421-424 行的独立 task 创建，只保留 Hook 注册。

**移除代码**：
```python
    if self_reflection:
        tasks.append(
            asyncio.create_task(self_reflection.reflection_loop(), name="morn-reflection")
        )
```

**注意**：保留第 388-393 行的 Hook 注册（事件驱动版本）。

---

## 任务 2：迁移 hindsight_loop 到事件驱动

**文件**：`morn_core/server.py` 和 `morn_core/eventbus/plugin_registry.py`

### 2a. 删除 hindsight_loop 函数定义

**文件**：`morn_core/server.py`

删除第 220-240 行的 `hindsight_loop` 函数定义。

**删除内容**：
```python
async def hindsight_loop(state: MornState):
    """每 1 小时检查是否有新的后见之明可触发，24小时内最多扫描一次"""
    while True:
        target_time = time.monotonic() + 3600
        if state.shutdown:
            break
        if hasattr(state, 'hindsight_engine') and state.hindsight_engine:
            try:
                emotion = state.chat_engine.emotion if state.chat_engine else None
                if emotion:
                    triggered = await state.hindsight_engine.tick(emotion)
                    if triggered:
                        state.log("hindsight", f"触发了 {len(triggered)} 条后见之明")
                        for h in triggered:
                            state.log("hindsight",
                                f"  memory#{h['memory_id']}: "
                                f"{h['original_tag']} → {h['new_tag']} "
                                f"({h['original_score']:.1f} → {h['new_score']:.1f})")
            except Exception as e:
                state.log("hindsight", f"tick failed: {e}")
        await asyncio.sleep(max(0, target_time - time.monotonic()))
```

### 2b. 移除 hindsight task 创建

**文件**：`morn_core/server.py`

删除第 426-429 行：
```python
    if hasattr(state, 'hindsight_engine') and state.hindsight_engine:
        tasks.append(
            asyncio.create_task(hindsight_loop(state), name="morn-hindsight")
        )
```

### 2c. 添加 hindsight Hook 注册

**文件**：`morn_core/eventbus/plugin_registry.py`

在末尾新增 `register_hindsight_hooks` 函数：

```python
def register_hindsight_hooks(event_bus, hook_manager, state):
    """后见之明：每小时检查一次"""
    async def on_hour(event):
        if not hasattr(state, 'hindsight_engine') or not state.hindsight_engine:
            return
        if not state.chat_engine:
            return
        try:
            emotion = state.chat_engine.emotion
            triggered = await state.hindsight_engine.tick(emotion)
            if triggered:
                await event_bus.publish(Event(
                    type="hindsight.triggered",
                    payload={"count": len(triggered)},
                    source="hindsight_engine",
                    priority=Priority.LOW,
                ))
        except Exception as e:
            await event_bus.publish(Event(
                type="task.failed",
                payload={"plugin": "hindsight_engine", "error": str(e)},
                source="hindsight_engine",
                priority=Priority.HIGH,
            ))

    hook_manager.register(HookRegistration(
        plugin_id="hindsight_engine",
        event_type="heartbeat.hour",
        callback=on_hour,
        timeout=30.0,
    ))
```

### 2d. 将 hindsight 注册加入 register_all_plugin_hooks

**文件**：`morn_core/eventbus/plugin_registry.py`

在 `register_all_plugin_hooks` 末尾追加 `register_hindsight_hooks(event_bus, hook_manager, state)`。

---

## 任务 3：验证

1. `python -c "from morn_core.server import main; print('server OK')"` — 导入正常
2. `python -m pytest tests/test_eventbus_integration.py -v` — 全部通过
3. `python -m pytest tests/ -x -q --timeout=120` — 无回归

---

## 验收标准

1. ✅ server.py 中没有 `hindsight_loop` 函数定义
2. ✅ server.py 中没有 `reflection_loop()` 的独立 task 创建
3. ✅ server.py 中没有 `hindsight_loop` 的 task 创建
4. ✅ `register_hindsight_hooks` 注册到 `heartbeat.hour`
5. ✅ `register_all_plugin_hooks` 包含 hindsight
6. ✅ 不破坏已有 EventBus 测试
