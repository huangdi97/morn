# 电脑操控文档

## 概览

`src/computer/` 目录实现电脑操控能力，包含 6 个模块：

| 模块 | 文件 | 功能 |
|------|------|------|
| 桌面操作 | `desktop_ops.rs` | 截图、窗口管理、鼠标键盘模拟 |
| 文件系统 | `fs_ops.rs` | 读写文件、目录操作、文件搜索 |
| 浏览器控制 | `browser_ops.rs` | 网页导航、元素点击、表单填写 |
| 应用管理 | `app_ops.rs` | 启动 / 关闭应用程序、进程管理 |
| 系统管理 | `sys_ops.rs` | 系统信息、环境变量、网络配置 |
| 感知 | `perception.rs` | 屏幕分析、OCR、音频输入 |

## 安全分级体系

`src/computer/mod.rs` 定义三级安全等级：

| 级别 | 含义 | 示例操作 |
|------|------|----------|
| L1Sandbox | 沙箱隔离 | 文件读写等受限操作 |
| L2Local | 本地操作 | 桌面截图、应用管理 |
| L3System | 系统级操作 | 系统配置修改 |

操作返回结果统一使用 `ComputerOpResult`：

```rust
pub struct ComputerOpResult {
    pub success: bool,
    pub data: String,
    pub security_level: String,
    pub approval_required: bool,
}
```

## 当前状态

所有电脑操控操作当前处于 **模拟阶段**。执行结果以模拟数据返回，不实际执行操作系统调用。

模拟行为：
- 所有调用返回 `success: true`
- `data` 字段返回 `[simulated]` 前缀的描述文本
- 操作用于验证架构和接口设计的正确性