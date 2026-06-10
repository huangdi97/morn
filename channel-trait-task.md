在 src/channel/mod.rs 文件末尾添加以下代码：

```rust
/// 统一 Channel 接口
pub trait Channel: Send + Sync {
    fn id(&self) -> &str;
    fn send(&self, msg: &str) -> Result<(), String>;
    fn receive(&self) -> Result<Option<String>, String>;
}
```

然后创建一个 channel_macro! 声明宏，接收一个 channel 名称参数（如 "telegram"），生成包含上述 trait 默认实现的 struct。

完成后运行 cargo build 验证。
