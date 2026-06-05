# Morn 创作台前端 ↔ 后端 — 任务清单

## 核心准则 (编程原则)

### 14条核心准则

1. **Think Before Coding** — 阅读整个任务文件再动手，理解现有前后端结构
2. **The Code Works** — 每次修改后 `cargo build` 通过，Tauri 构建通过
3. **Small Batches** — 每完成一个任务的改动就验证编译
4. **No Dead Code** — 不添加未使用的 Tauri 命令或前端组件
5. **Single Source of Truth** — 前端 API 类型与后端 Rust 结构体保持一致
6. **Test the Paths** — Tauri 命令添加后确保编译通过
7. **Fail Fast** — 遇到编译/类型错误立即停止
8. **Leave It Better** — 前端风格与现有 App.tsx 保持一致（暗色主题、GitHub 色调）
9. **Never Guess the Stack** — 从实际代码中理解 Tauri invoke 接口
10. **Read Before You Write** — 先读 src-tauri/src/lib.rs、web/src/App.tsx 等关键文件
11. **Prefer Friction Logs** — 记录构建中遇到的问题
12. **Respect the Dependency** — 前端依赖 @tauri-apps/api（已安装）
13. **No Ambiguous Names** — Tauri 命令命名统一 camelCase
14. **Document Decisions, Not Drama** — 写清晰的代码注释

### 低耦合3条

1. **视图独立** — 工作台/创作台/管理台三个视图互相独立，通过导航切换
2. **命令独立** — 每个 Tauri 命令做一件事，不搞大而全
3. **前端状态本地化** — 各组件管理自己的 state，不搞全局状态管理

### 执行规则

- 按任务顺序执行
- 每个任务后 `cargo build` 验证
- 以 `~/morn-desktop/` 为根目录
- 前端文件在 `web/src/` 下
- 后端 Tauri 命令在 `src-tauri/src/lib.rs` 中
- 最终 git add/commit/push 到 main

---

## 前置阅读

- `src-tauri/src/lib.rs` — 现有 Tauri 命令
- `src-tauri/Cargo.toml` — Tauri 依赖
- `web/src/App.tsx` — 主页面（工作台）
- `web/src/App.css` — 样式
- `web/vite.config.ts` — Vite 配置
- `web/package.json` — 前端依赖
- `src/studio/manager.rs` — 创作台后端
- `src/studio/tester.rs` — 测试器
- `src/studio/publisher.rs` — 发布器
- `src/console/mod.rs` — 管理台后端

---

## 任务列表

### 任务1: Tauri 后端 — 注册创作台命令

在 `src-tauri/src/lib.rs` 中新增以下 Tauri 命令：

**创作台组件管理：**
- `list_components(type_filter: Option<String>, state)` — 列出组件
- `get_component(id: String, state)` — 获取组件详情
- `create_component(name: String, component_type: String, config_json: Option<String>, state)` — 创建组件
- `update_component(id: String, name: Option<String>, config_json: Option<String>, status: Option<String>, state)` — 更新组件
- `delete_component(id: String, state)` — 删除组件

**Agent 组装：**
- `assemble_agent(name: String, persona: String, model: String, tools: Vec<String>, knowledge: Vec<String>, skills: Vec<String>, state)` — 从组件组装 Agent

**测试与发布：**
- `test_component(id: String, input: String, state)` — 测试组件
- `publish_component(id: String, state)` — 发布组件到市场

**管理台：**
- `get_system_status(state)` — 系统健康状态
- `get_component_topology(state)` — 组件拓扑

需要：
- 将 StudioManager、StudioPublisher、StudioTester 加入 AppState
- 在 run() 中创建并注入这些实例
- 注册所有新命令到 invoke_handler

注意：
- 所有命令返回 `Result<serde_json::Value, String>` 以保持统一
- 使用 `tokio::runtime::Runtime::new()` 包裹异步调用（如 chat_agent）
- 保持与现有 `send_message`、`get_status`、`clear_history` 命令风格一致

### 任务2: 前端导航栏

修改 `web/src/App.tsx`：

添加顶部导航栏，三个标签页：
- **💬 工作台 (Workbench)** — 现有聊天界面
- **🔧 创作台 (Studio)** — 组件编辑 + Agent 组装 + 测试
- **📊 管理台 (Console)** — 拓扑图 + 系统状态

导航逻辑：
- 使用 `useState<"workbench" | "studio" | "console">("workbench")` 控制当前视图
- 导航栏样式与现有暗色主题一致（GitHub 色调）
- 导航项点击切换视图

### 任务3: 创作台前端 — ComponentEditor 接线

修改 `web/src/studio/ComponentEditor.tsx`：

将 `handleSave` 改为真实调用 Tauri 后端：
```typescript
const handleSave = async () => {
  const id = await invoke("create_component", {
    name: def.name,
    componentType: def.type,
    configJson: def.config,
  });
  setSaved(true);
  setLastCreatedId(id);
  setTimeout(() => setSaved(false), 2000);
};
```

需添加：
- `import { invoke } from "@tauri-apps/api/core";`
- 保存后清空表单或显示成功状态
- 类型字段保持与后端一致

### 任务4: 创作台前端 — AgentBuilder 接线

修改 `web/src/studio/AgentBuilder.tsx`：

将 build 按钮改为调用 `assemble_agent`：
```typescript
const handleBuild = async () => {
  const id = await invoke("assemble_agent", {
    name: def.name,
    persona: def.persona,
    model: def.model,
    tools: def.tools,
    knowledge: def.knowledge,
    skills: def.skills,
  });
  setAgentId(id);
  setStep(2); // show success
};
```

添加：
- `import { invoke } from "@tauri-apps/api/core";`
- 成功后的 Agent ID 显示
- 错误处理

### 任务5: 创作台前端 — TestPanel 接线

修改 `web/src/studio/TestPanel.tsx`：

将 Test 按钮改为调用 `test_component`，获取真实测试结果展示。

### 任务6: 创作台前端 — TemplateSelector

修改 `web/src/studio/TemplateSelector.tsx`：

从 `list_components()` 加载可用组件列表，而不是硬编码。

### 任务7: 管理台前端接线

修改 `web/src/console/Topology.tsx`：

从后端调用 `get_component_topology` 获取实时组件拓扑，而不是硬编码。

调用 `get_system_status` 显示真实系统状态。

### 任务8: 样式整合

更新 `web/src/App.css`：

- 添加导航栏样式（标签页切换）
- 添加创作台表单样式
- 添加管理台表格/拓扑样式
- 确保所有视图在暗色主题下一致

### 任务9: 编译验证

```bash
cd ~/morn-desktop/src-tauri && cargo build 2>&1
cd ~/morn-desktop/web && npm install --silent && npx tsc --noEmit 2>&1 || echo "TypeScript check done (non-blocking if no tsc config)"
```

### 任务10: Git 提交与推送

```bash
cd ~/morn-desktop
git add -A
git commit -m "feat(studio): connect frontend to backend via Tauri commands

- Add 8 Tauri commands for studio/console operations
- Add navigation (workbench/studio/console)
- Wire ComponentEditor to create_component backend
- Wire AgentBuilder to assemble_agent backend
- Wire TestPanel to test_component backend
- Wire Topology to get_component_topology backend"
git push origin main
```
