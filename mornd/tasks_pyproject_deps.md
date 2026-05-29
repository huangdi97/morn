# 任务：拆分 pyproject.toml 依赖

将 pyproject.toml 的重型外部依赖拆分为可选 extras。

## 当前依赖

```
dependencies = [
    "aiohttp>=3.9",
    "aiogram>=3.0",
    "aiosqlite>=0.20",
    "chromadb>=0.5.0",
    "ollama>=0.3",
    "pydantic>=2.0",
    "pycryptodome>=3.20",
    "psutil>=5.9",
    "matplotlib>=3.8",
]
```

## 修改后

```
# 核心依赖（所有实例必须）
dependencies = [
    "aiohttp>=3.9",
    "aiosqlite>=0.20",
    "pydantic>=2.0",
    "pycryptodome>=3.20",
    "psutil>=5.9",
]

[project.optional-dependencies]
telegram = ["aiogram>=3.0"]
local-llm = ["ollama>=0.3"]
vector = ["chromadb>=0.5.0"]
all = ["morn-core[telegram,local-llm,vector]"]
```

注意：保留 `[project.optional-dependencies]` 下的 test / redis / dev 分组，新增 telegram / local-llm / vector / all。

文件路径：/home/hermes/morn/mornd/pyproject.toml
