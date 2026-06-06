# Morn 电脑操控与高级能力

> 桌面操控 · 文件系统 · 浏览器 · 视觉 GUI · Office · 3D 可视化

## 电脑操控 (Computer Control)

### 6 大操控模块

| 模块 | 功能 |
|------|------|
| 桌面操作 | 窗口管理、截图、按键模拟 |
| 文件系统 | 文件/文件夹 CRUD、搜索 |
| 浏览器控制 | 页面导航、内容提取、表单填写 |
| 应用管理 | 启动/关闭应用、进程列表 |
| 系统管理 | 系统设置、环境变量 |
| 感知模块 | 屏幕内容理解（辅助视觉GUI） |

### 安全限制
| 操作类型 | 安全等级 |
|---------|---------|
| 读文件 | L4 (自由) |
| 写文件 | L2 (需通知) |
| 修改系统 | L1 (需审批) |
| 执行命令 | L1 (需审批) |

## 视觉 GUI 操控 (VisualAgent)

VLM 驱动的 GUI 理解与操作：

```rust
pub struct VisualAgent {
    // 接收截图 → UI 元素检测 → 坐标操作
}

impl VisualAgent {
    pub fn detect_buttons(&self, screenshot: &[u8]) -> Vec<Button>;
    pub fn detect_text_fields(&self, screenshot: &[u8]) -> Vec<TextField>;
    pub fn detect_images(&self, screenshot: &[u8]) -> Vec<ImageElement>;
    pub fn click_at(&self, x: u32, y: u32);
    pub fn type_at(&self, text: &str, x: u32, y: u32);
}
```

**工作流程：**
1. 截取当前屏幕
2. VLM 分析 UI 元素位置和类型
3. 识别用户目标对应的操作坐标
4. 执行点击/输入操作
5. 验证结果

## Office 文档处理 (OfficeHandler)

纯 Rust 方案，无需安装 Office：

### PPT 生成
```rust
pub fn create_slide_from_template(template: &str, data: &Value) -> Slide;
pub fn export_to_pptx(slides: &[Slide], path: &str) -> Result<()>;
```

### Excel 导出
```rust
pub fn export_to_csv(data: &[Vec<String>], path: &str) -> Result<()>;
pub fn export_to_xlsx(data: &[Vec<String>], path: &str) -> Result<()>;
```

| 功能 | 依赖 |
|------|------|
| PPT 生成 | zip + serde_json |
| CSV 导出 | csv crate |
| XLSX 导出 | rust_xlsxwriter |

## 3D 可视化仪表盘 (Visualization3D)

Agent 调用链和工作流路径的 3D 力导向图数据：

```rust
pub struct Graph3D {
    pub nodes: Vec<Node3D>,
    pub edges: Vec<Edge3D>,
}

pub struct Node3D {
    pub id: String,
    pub label: String,
    pub node_type: NodeType,  // Agent / Task / DataFlow
    pub x: f32, pub y: f32, pub z: f32,
}

pub struct Edge3D {
    pub source: String,
    pub target: String,
    pub weight: f32,
    pub edge_type: EdgeType,  // Call / DataFlow
}
```

输出格式兼容 Three.js / react-three-fiber。

## PikoSoul 性格引擎 (PersonalityEngine)

五维性格分析：

```rust
pub struct PersonalityProfile {
    pub openness: f32,      // 开放性 (0-1)
    pub conscientiousness: f32, // 尽责性 (0-1)
    pub extraversion: f32,  // 外向性 (0-1)
    pub agreeableness: f32, // 宜人性 (0-1)
    pub neuroticism: f32,   // 神经质 (0-1)
}
```

| 用途 | 描述 |
|------|------|
| Agent 对话风格 | 根据上传历史调整语气 |
| 团队角色匹配 | 推荐互补性格的 Agent |
| 用户画像 | 分析使用者偏好 |

## PC Tracker 认知录制 (DemoRecorder)

录制用户操作过程：

```rust
pub struct DemoRecording {
    pub id: String,
    pub name: String,
    pub steps: Vec<RecordingStep>,
    pub duration: Duration,
    pub created_at: DateTime,
}

pub struct RecordingStep {
    pub action: String,      // "click", "type", "scroll"
    pub target: String,      // "button#submit"
    pub timestamp: DateTime,
    pub screenshot: Option<Vec<u8>>,
}
```

支持回放和导出为工作流模板。
