# Batch 4 — 语音输入修复 + Whisper 集成

修 VoiceInput 的 blob URL 路径崩溃，把麦克风按钮接进输入栏。

## 任务列表

### T1: 修 VoiceInput blob URL 路径

文件：`web/src/components/VoiceInput.tsx`

当前问题：
```tsx
// VoiceInput 录制完成后：
const blob = new Blob(chunks, { type: 'audio/webm' });
const url = URL.createObjectURL(blob);  // blob:http://localhost:1420/xxx
invoke('transcribe_audio', { path: url });  // 后端 Path::new(url).exists() → false!
```

修复方案：
1. 将 blob 转为 ArrayBuffer
2. 通过 Tauri 自定义协议或临时文件写入传递录音数据
3. 或在 Tauri 命令中接受 base64 编码的音频数据代替文件路径

**推荐方案：** 在后端 `transcribe_audio` 命令增加过载，接受 base64 编码的音频数据：
```rust
#[tauri::command]
pub(crate) fn transcribe_audio(
    path: Option<String>,
    data: Option<String>,  // base64 编码的音频数据
) -> Result<String, CommandError> {
    let audio_bytes = if let Some(data) = data {
        base64::decode(&data).map_err(|e| ...)?
    } else if let Some(path) = path {
        std::fs::read(&path).map_err(|e| ...)?
    } else {
        return Err("No audio data provided".into());
    };
    // 将字节写入临时文件
    let tmp_path = std::env::temp_dir().join(format!("morn_voice_{}.webm", uuid::Uuid::new_v4()));
    std::fs::write(&tmp_path, &audio_bytes).map_err(|e| ...)?;
    // 调 whisper CLI
    let output = std::process::Command::new("whisper")
        .arg(&tmp_path)
        .arg("--output_format")
        .arg("txt")
        .output()
        .map_err(|e| ...)?;
    // ...
}
```

前端 VoiceInput 改为发送 base64：
```tsx
const reader = new FileReader();
reader.onload = () => {
    const base64 = (reader.result as string).split(',')[1]; // 去掉 data:audio/webm;base64, 前缀
    invoke('transcribe_audio', { data: base64 }).then(...);
};
reader.readAsDataURL(blob);
```

### T2: 麦克风按钮接入输入栏

文件：`web/src/App.tsx`

当前：`VoiceInput` 被 import 了但从未在输入栏中渲染。

在输入框旁边加麦克风按钮：
```tsx
<div className="input-area">
  <VoiceInput onTranscribed={(text) => setInput(text)} />
  <textarea value={input} onChange={...} onKeyDown={...} />
  <button onClick={sendMessage}>Send</button>
</div>
```

需要添加 `VoiceInput` 的 CSS 样式（`web/src/styles/chat.css`）：
```css
.voice-input-btn { ... }
.voice-input-btn.recording { ... }
```

### T3: 音频面板设备列表

文件：`src-tauri/src/commands/whisper.rs`

当前 `list_audio_devices` 返回硬编码 `["default"]`。

改为扫描系统音频设备（或至少返回更真实的列表）：
- Linux: `arecord -l` 输出
- 保留 `["default"]` 作为保底
- 如果 `arecord` 可用，解析其输出

### T4: 验证

- `npm run build` ✅
- `tsc --noEmit` ✅
- 输入栏显示麦克风按钮
- 点击开始录音 → 停止 → 转写 → 结果填入输入框
- 无 blob URL 路径错误
