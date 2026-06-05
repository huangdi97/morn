# Morn 剩余五方向 — warning清理 + 前端完善 + 三渠道做真 + A2A + 跨设备同步

## 核心准则
14条核心准则 + 低耦合3条 + 执行规则（每任务 cargo build && cargo test，git push 到 main）

---

## 前置阅读
现有代码已完成 Tauri 前后端接线 + 14 个前端页面，需评估 UI 质量和完整度。
现有 Telegram/SMTP/REST API 已做真，钉钉/飞书/企维持追加。

---

## 阶段一：清理 46 个 warning（耗时小，收益大）

### 任务1: 自动修复可修 warning
```bash
cd ~/morn-desktop && cargo fix --allow-dirty 2>&1 | tail -5
```
然后用 `cargo build 2>&1 | grep "warning:" | wc -l` 计数剩余。

### 任务2: 手工修剩余 warning
逐一处理剩余的 unused variable / dead_code：
- `supervisor.rs` — unused import `EVENT_TASK_FAILED`
- `assembler.rs` — unused imports Knowledge/Memory/Skill/Tool
- `tool.rs` — `name` 字段标记 `#[allow(dead_code)]` 或加 `_` 前缀
- `channel/wecom.rs`, `dingtalk.rs`, `feishu.rs` — `msg` 参数加 `_` 前缀
- 其余类似模式

原则：不删字段（未来要用），加 `#[allow(dead_code)]` 或 `_` 前缀。

---

## 阶段二：三渠道做真（钉钉/飞书/企微）

三个渠道都支持 webhook 模式发送消息，无需 bot token，只需 webhook URL。

### 任务3: 钉钉做真
改造 `src/channel/dingtalk.rs`：

**发送消息：**
- 读取 `DINGTALK_WEBHOOK_URL` 环境变量
- POST JSON 到钉钉 webhook URL
- 钉钉 webhook 格式：`{"msgtype": "text", "text": {"content": "消息内容"}}`
- 使用 reqwest 的 blocking 模式（已添加依赖）
- 错误处理：连接超时、HTTP 非 200、JSON 解析错误

### 任务4: 飞书做真
改造 `src/channel/feishu.rs`：

**发送消息：**
- 读取 `FEISHU_WEBHOOK_URL` 环境变量
- POST JSON 到飞书 webhook URL
- 飞书 webhook 格式：`{"msg_type": "text", "content": {"text": "消息内容"}}`
- 使用 reqwest blocking
- 错误处理同上

### 任务5: 企微做真
改造 `src/channel/wecom.rs`：

**发送消息：**
- 读取 `WECOM_WEBHOOK_URL` 环境变量
- POST JSON 到企业微信 webhook URL
- 企微 webhook 格式：`{"msgtype": "text", "text": {"content": "消息内容"}}`
- 使用 reqwest blocking
- 错误处理同上

### 任务6: 测试
为三个渠道各写一个单元测试，验证 HTTP 请求构造正确（不实际发送，使用 mock 或仅构造请求体验证）。

---

## 阶段三：前端 UI 完善

现有 14 个 TSX 文件已有完整页面 + Tauri 调用，重点是补齐功能、美化样式。

### 任务7: 工作台聊天完善
改造 `web/src/App.tsx`：
- 添加聊天记录的持久化（LocalStorage 或 Tauri invoke 存到 SQLite）
- 添加消息时间戳显示
- 添加打字中状态指示器
- 添加/clear 命令的前端清空
- 主题切换（浅色/深色 toggle）

### 任务8: 创作台完善
改造 `web/src/studio/ComponentEditor.tsx` + `AgentBuilder.tsx`：

ComponentEditor：
- 从 `list_components` 加载已有组件列表并显示在侧边栏
- 编辑已有组件（调用 `get_component` + `update_component`）
- 保存后刷新列表
- 删除组件（调用 `delete_component`）

AgentBuilder：
- 从 `list_components` 加载可用工具/知识/技能下拉列表
- 组装后展示 Agent ID 和组件清单
- 发布到工作台按钮

### 任务9: 管理台完善
改造 `web/src/console/` 下所有文件：
- Dashboard：展示真实数据（组件数、Agent 数、用户数、团队数）
- SystemInfo：从 `get_system_status` 加载实时信息
- Topology：从 `get_component_topology` 加载
- CostCenter：从 `get_system_status` 加载成本数据
- Governance：显示策略列表
- Security：显示安全策略
- Marketplace：从市场加载商品列表

### 任务10: 前端样式统一
改造 `web/src/App.css`：
- 统一所有页面的暗色主题样式（卡片、表格、按钮、输入框）
- 响应式布局（适配不同窗口大小）
- 动画过渡效果
- 滚动条美化

### 任务11: 编译验证
```bash
cd ~/morn-desktop/web && npm install && npx tsc --noEmit 2>&1
```

---

## 阶段四：A2A 协议（Agent-to-Agent）

A2A 让不同机器上的 Morn Agent 能互相发现、通信、协作。

### 任务12: A2A 数据模型新建 `src/bridge/a2a.rs`

```rust
pub struct AgentCard {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub capabilities: Vec<String>,
    pub endpoint: String,
    pub public_key: String,
}
pub enum A2AMessage {
    TaskAssign { task_id: String, input: String, max_tokens: u32 },
    TaskStatus { task_id: String, status: String, progress: f64 },
    TaskResult { task_id: String, output: String, success: bool },
    AgentDiscovery { query: String },
    AgentList { agents: Vec<AgentCard> },
    Heartbeat,
    Error { code: u32, message: String },
}
pub struct A2AProtocol;
impl A2AProtocol {
    pub fn serialize(msg: &A2AMessage) -> Result<String, String>;
    pub fn deserialize(data: &str) -> Result<A2AMessage, String>;
    pub fn send(endpoint: &str, msg: &A2AMessage) -> Result<A2AMessage, String>;
}
```

### 任务13: A2A 发现服务
新建 `src/bridge/a2a_discovery.rs`：
- 局域网 mDNS 广播发现其它 Morn 实例
- Agent 能力列表交换
- 定时心跳检查

### 任务14: A2A 调度集成
改造 `src/core/orchestrator.rs`：
- 当任务需要远程 Agent 时，通过 A2A 发送 TaskAssign
- 轮询 TaskStatus
- 接收 TaskResult
- 远程 Agent 作为团队的成员参与协作

---

## 阶段五：跨设备同步

### 任务15: 同步数据模型
在 `src/core/storage.rs` 新增表：
- `sync_events` — id, entity_type, entity_id, action, data_json, timestamp, device_id, synced
- `devices` — id, name, last_seen, public_key

### 任务16: 同步引擎
新建 `src/bridge/sync.rs`：
- `SyncEngine` — 管理同步队列
- `push_changes()` — 将本地变更推送到远程
- `pull_changes()` — 从远程拉取变更
- `resolve_conflicts()` — 简单冲突解决（last-write-wins）

### 任务17: 同步集成
- Storage 写入时自动产生 sync event
- COO Supervisor 执行完任务后触发同步
- 同步服务器 URL 通过 `SYNC_SERVER_URL` 环境变量配置

---

## 最终验证

### 任务18: 全量编译与测试
```bash
cd ~/morn-desktop
# 计数 warning
cargo build 2>&1 | grep "warning:" | wc -l
# 全量测试
cargo test 2>&1 | grep "test result"
# TypeScript 检查
cd web && npx tsc --noEmit 2>&1 | tail -5
```

### 任务19: Git 提交
```bash
cd ~/morn-desktop
git add -A
git commit -m "feat: clean warnings + frontend polish + real channels + A2A + sync"
git push origin main
```
