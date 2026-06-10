把以下 Rust 代码原样添加到 src/channel/mod.rs 文件末尾（最后一个 pub mod 声明之后），不要修改任何现有代码：

```rust
/// 统一 Channel 接口
pub trait Channel: Send + Sync {
    fn id(&self) -> &str;
    fn send(&self, msg: &str) -> Result<(), String>;
    fn receive(&self) -> Result<Option<String>, String>;
}

/// 生成 Channel 实现的宏
/// 用法: channel_macro!(Telegram);
#[macro_export]
macro_rules! channel_macro {
    ($name:ident) => {
        pub struct $name;
        impl Channel for $name {
            fn id(&self) -> &str {
                stringify!($name)
            }
            fn send(&self, msg: &str) -> Result<(), String> {
                println!("[{}] 发送: {}", stringify!($name), msg);
                Ok(())
            }
            fn receive(&self) -> Result<Option<String>, String> {
                Ok(None)
            }
        }
    };
}
```

添加后运行 `cargo build` 验证。
