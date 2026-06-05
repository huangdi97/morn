# 渠道适配文档

## 统一消息适配器

`src/channel/adapter.rs::ChannelAdapter` 是消息处理的中枢：

```rust
pub struct ChannelMessage {
    pub content: String,
    pub source: String,       // 来源标识（"cli", "telegram", ...）
    pub timestamp: i64,
    pub metadata: Value,
}
```

`ChannelAdapter::handle_message(msg)` 将消息路由到 Supervisor 处理，返回响应文本。

## 已支持的渠道

`src/channel/` 目录下包含 12 个渠道实现：

| 文件 | 渠道 | 说明 |
|------|------|------|
| `cli.rs` | CLI | 标准输入 / 输出 REPL |
| `telegram.rs` | Telegram Bot | 通过 Telegram Bot API 收发消息 |
| `wecom.rs` | 企业微信 | 企业微信机器人消息处理 |
| `dingtalk.rs` | 钉钉 | 钉钉机器人消息处理 |
| `feishu.rs` | 飞书 | 飞书机器人消息处理 |
| `rest_api.rs` | REST API | HTTP RESTful API 接口 |
| `qqbot.rs` | QQ Bot | QQ 机器人消息处理 |
| `miniprogram.rs` | 微信小程序 | 微信小程序消息处理 |
| `wechat_mp.rs` | 微信公众号 | 微信公众号消息处理 |
| `webhook.rs` | Webhook | 通用 Webhook 接收 |
| `smtp.rs` | SMTP | 邮件收发 |
| `desktop.rs` | Desktop | Tauri 桌面端通信 |

## 如何添加新渠道

1. 在 `src/channel/` 下新建文件 `your_channel.rs`
2. 实现消息接收和发送逻辑
3. 在 `src/channel/mod.rs` 注册模块
4. 通过 `ChannelAdapter` 处理消息：
   ```rust
   let msg = ChannelMessage::new("用户输入", "your_channel");
   let response = adapter.handle_message(&msg);
   ```
5. 渠道配置和密钥管理通过 `Console` 的治理模块进行