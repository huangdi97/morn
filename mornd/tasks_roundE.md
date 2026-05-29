# Morn — 轮E：语音 + L3审计LLM接入

## 模块E1：语音识别（Vosk 离线）

### 新文件
- `morn_core/media/speech_recognition.py`

### 规格
SpeechRecognizer 类：

`transcribe(audio_path)` — 将音频文件转为文字
`is_available()` — 检查Vosk模型是否已安装
`get_languages()` — 返回支持的语言列表
`download_model(lang='zh-cn')` — 下载Vosk模型

配置项：
- enabled（默认False）
- model_path（默认 ~/.morn/models/vosk/）
- language（默认 zh-cn）

默认禁用（需创建者下载模型后启用）。

### 新增测试
`tests/test_speech_recognition.py` — 6+测试
- 默认禁用
- 模型不存在时优雅报错
- 语言切换

---

## 模块E2：语音合成（Piper / edge-tts）

### 新文件
- `morn_core/media/speech_synthesis.py`

### 规格
SpeechSynthesizer 类：

`speak(text, voice=None)` — 将文字转为语音文件并返回路径
`is_available()` — 检查引擎是否可用
`get_available_voices()` — 返回可用语音列表
`set_engine(engine)` — 切换引擎（piper / edge-tts / none）

配置项：
- enabled（默认False）
- engine（默认 edge-tts）
- voice（默认 zh-CN-XiaoxiaoNeural）

默认禁用。

### 新增测试
`tests/test_speech_synthesis.py` — 6+测试
- 默认禁用
- 引擎切换
- 可用性检测

---

## 模块E3：L3审计回路接入LLM

### 修改文件
- `morn_core/memory/audit_agent.py`

### 规格
升级 AuditAgent 的审计逻辑：

当前：规则引擎模拟审计（基于关键词和简单规则）
升级后：调用LLM做三元组审核

`_llm_audit(triple, source_text)` — 调LLM判断三元组是否准确
- prompt：提供源文本和三元组，要求LLM返回 "pass" / "fail" / "uncertain"
- 使用 ChatEngine 的 LLM 路由（云端→本地兜底）
- LLM不可用时回退到规则引擎

`_llm_extract(capsule_text)` — 调LLM从文本中提取三元组
- 替代当前的关键词提取
- LLM不可用时回退到关键词提取

配置项：
- llm_audit_enabled（默认True）— 开启LLM审计，失败时静默回退规则引擎
- llm_extract_enabled（默认True）

### 新增测试
`tests/test_audit_agent_llm.py` — 8+测试
- LLM审计通过
- LLM审计不通过
- LLM不可用时回退规则引擎
- LLM提取三元组
- 配置可禁用LLM

---

## 验收
1. `python3 -m pytest tests/ -q` — 全部通过
2. 语音模块默认禁用不影响功能
3. L3审计LLM不可用时回退规则引擎
