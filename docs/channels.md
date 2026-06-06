# Morn 渠道与通信

> CLI · 企微 · 钉钉 · 飞书 · Telegram · 微信 · REST API · 邮件

## 渠道架构

```
                    ┌──────────────────┐
                    │ ChannelAdapter   │
                    │ (统一消息适配器)  │
                    └────────┬─────────┘
                             │
          ┌──────────────────┼──────────────────┐
          │                  │                   │
   ┌──────▼──────┐  ┌───────▼───────┐  ┌───────▼───────┐
   │  CLI 信道    │  │ IM 信道群     │  │ API 信道      │
   │  (stdin/out) │  │ 企微/钉钉/    │  │ REST/Webhook/ │
   │              │  │ 飞书/Telegram │  │ SMTP          │
   └─────────────┘  └───────────────┘  └───────────────┘
```

## 支持渠道

| 渠道 | 状态 | 传输协议 |
|------|------|---------|
| CLI (本地终端) | ✅ | stdin/stdout REPL |
| REST API | ✅ | HTTP JSON (axum) |
| SMTP 邮件 | ✅ | SMTP (lettre) |
| 企业微信 | ✅ | HTTP Webhook / Bot API |
| 钉钉 | ✅ | HTTP Webhook / Bot API |
| 飞书 | ✅ | HTTP Webhook / Bot API |
| 微信小程序 | ✅ | HTTP JSON |
| Telegram | ✅ | Bot API |
| QQ Bot | ✅ | HTTP JSON |
| 微信公众号 | ✅ | HTTP XML/JSON |
| 浏览器扩展 | ✅ | WebSocket |

## 跨渠道身份统一 (IdentityBridge)

```rust
pub struct IdentityBridge {
    // Telegram 用户 A → Morn 用户 1
    // 企业微信 用户 B → Morn 用户 1
    // 同一个人的不同渠道绑定为统一身份
}
```

| 功能 | 描述 |
|------|------|
| 身份绑定 | 多渠道用户合并为统一身份 |
| 会话同步 | 跨渠道保持同一对话上下文 |
| 通知路由 | 按渠道优先级推送通知 |

## REST API 端点

axum 服务器 (默认端口 3000)：

| 方法 | 路径 | 描述 |
|------|------|------|
| POST | `/api/chat` | 发送消息 |
| GET | `/api/status` | 系统状态 |
| GET | `/api/tasks` | 任务列表 |
| POST | `/api/components` | 创建组件 |
| GET | `/api/components` | 列出组件 |
| GET | `/api/components/:id` | 组件详情 |

## 消息格式

```json
{
    "role": "user | assistant",
    "content": "消息文本",
    "channel": "cli | wecom | dingtalk | feishu",
    "timestamp": 1717000000
}
```
