# Batch 2 — 本地模型推理引擎

真实化本地模型 Tauri 命令 + 推理链路。当前 Tauri 命令返回硬编码值，需要改为真实文件系统操作。

## 编码准则

1. 【Think Before Coding】先读现有代码再改
2. 【增量叠加】不更改现有 ModelRouter 路由逻辑，只补真实操作的实现
3. 【最小改动】不改前端 LocalModelPanel.tsx（它已经正确调用命令）
4. 【错误处理】所有文件操作必须 Result 返回，不允许 unwrap

## 任务列表

### T1: 真实化 list_local_models 命令

文件：`src-tauri/src/commands/local_model.rs`

当前：
```rust
fn list_local_models() -> Result<Vec<String>, CommandError> {
    Ok(vec!["llama-3.2-3b".to_string(), "qwen-2.5-7b".to_string()])
}
```

改为：
1. 扫描 `~/.morn/models/` 目录（不存在则返回空列表）
2. 查找所有 `.gguf` 文件
3. 对每个文件解析文件名（去掉 `.gguf` 后缀）作为模型名称
4. 返回 `Vec<String>` 模型名列表
5. 需要访问 AppState 吗？不需要，纯文件系统操作

```rust
use std::path::PathBuf;
use tauri::api::path::home_dir;

fn get_models_dir() -> PathBuf {
    let mut path = home_dir().unwrap_or_else(|| PathBuf::from("~"));
    path.push(".morn");
    path.push("models");
    path
}

fn list_local_models() -> Result<Vec<String>, CommandError> {
    let models_dir = get_models_dir();
    if !models_dir.exists() {
        return Ok(vec![]);
    }
    let mut models = Vec::new();
    for entry in std::fs::read_dir(&models_dir).map_err(|e| CommandError::from(e.to_string()))? {
        let entry = entry.map_err(|e| CommandError::from(e.to_string()))?;
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "gguf") {
            if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                models.push(name.to_string());
            }
        }
    }
    models.sort();
    Ok(models)
}
```

### T2: 实化下载 download_model 命令

当前返回 `Ok(format!("downloading {name}"))`。

改为通过 HTTP 从 HuggingFace 或指定 URL 下载 GGUF 文件：
1. 检查模型是否已存在（避免重复下载）
2. 使用 reqwest（Cargo.toml 已有）下载文件到 `~/.morn/models/{name}.gguf`
3. 返回 "ok" 或错误信息
4. 简版实现：先支持从已知 HuggingFace mirror URL 下载（如 `https://huggingface.co/{org}/{repo}/resolve/main/{name}.gguf`）
5. 支持指定自定义 URL

```rust
#[tauri::command]
fn download_model(name: String, url: Option<String>) -> Result<String, CommandError> {
    let models_dir = get_models_dir();
    std::fs::create_dir_all(&models_dir).map_err(|e| CommandError::from(e.to_string()))?;
    
    let model_path = models_dir.join(format!("{}.gguf", name));
    if model_path.exists() {
        return Ok(format!("already exists: {}", model_path.display()));
    }
    
    // 确定下载 URL
    let download_url = url.unwrap_or_else(|| {
        format!("https://huggingface.co/{}/resolve/main/{}.gguf", 
            get_model_hf_repo(&name), name)
    });
    
    // 使用 reqwest 下载
    // ...
}
```

辅助函数 `get_model_hf_repo(name)` 维护一个已知模型名称到 HF 仓库的映射。

### T3: 实化 delete_local_model 命令

当前返回 `Ok(format!("deleted {name}"))`。

改为：
```rust
fn delete_local_model(name: String) -> Result<String, CommandError> {
    let model_path = get_models_dir().join(format!("{}.gguf", name));
    if !model_path.exists() {
        return Err(CommandError::from(format!("Model '{}' not found", name)));
    }
    std::fs::remove_file(&model_path)
        .map_err(|e| CommandError::from(format!("Failed to delete: {}", e)))?;
    Ok(format!("deleted {}", model_path.display()))
}
```

### T4: 集成 Ollama/LMStudio 检测

当前 `src/bridge/local_model.rs` 已有调用 localhost:11434 的 OpenAI 兼容客户端。

在 `list_local_models` 中增加：
1. 尝试连接 `http://localhost:11434/api/tags`（Ollama）
2. 如果通了，将 Ollama 上的模型也加入列表（标注 `ollama/` 前缀）
3. 同样尝试 `http://localhost:1234/v1/models`（LM Studio）

### T5: LocalModelPanel UI 适配

检查 `web/src/console/LocalModelPanel.tsx` 是否已能正确处理真实返回值。
如果命令返回空列表/真实列表，UI 应正确渲染。不需要改动 UI 代码。

### T6: 验证

```bash
# 创建测试目录和文件
mkdir -p ~/.morn/models
touch ~/.morn/models/llama-3.2-3b.Q4_K_M.gguf
touch ~/.morn/models/qwen-2.5-7b.Q4_K_M.gguf

# 编译
cargo check -p morn
```

## 验证门禁

- `cargo check -p morn` ✅
- `cargo test --lib` 全部通过
- `cargo clippy -p morn` 0 warnings
- `list_local_models` 返回真实 GGUF 文件列表
- `delete_local_model` 实际删除文件
