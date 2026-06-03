"""Morn CLI Presence — 默认对话界面"""
import asyncio
import sys
import time

from morn.sdk.presence import MornPresence


def _welcome(state: dict):
    preset = state.get("preset", "平衡型")
    model = state.get("model", "hybrid")
    instance = state.get("instance", "default")
    print()
    print("  Morn — 数字生命框架")
    print(f"  实例: {instance}")
    print(f"  预设: {preset}")
    print(f"  模型: {model}")
    if state.get("telegram"):
        print("  Telegram: 已集成")
    print("  配置文件: ~/.morn/config.yaml")
    print()
    print("  输入 /status 查看状态，/shutdown 退出")


def cmd_init(args):
    import json
    from pathlib import Path

    config_dir = Path.home() / ".morn"
    config_dir.mkdir(parents=True, exist_ok=True)
    config_path = config_dir / "config.yaml"

    print("=== Morn 初始化 ===")
    print()

    presets = {"1": "最小系统", "2": "高效率", "3": "高情感"}
    print("请选择预设方案：")
    for k, v in presets.items():
        print(f"  {k}. {v}")
    preset_key = input("请输入编号 (1-3，默认 2): ").strip() or "2"
    preset = presets.get(preset_key, "高效率")

    models = {"1": "deepseek", "2": "ollama", "3": "anthropic", "4": "openai"}
    print("\n请选择模型：")
    for k, v in models.items():
        print(f"  {k}. {v}")
    model_key = input("请输入编号 (1-4，默认 1): ").strip() or "1"
    model = models.get(model_key, "deepseek")

    api_key = ""
    if model in ("deepseek", "anthropic", "openai"):
        api_key = input(f"\n请输入 {model} API key (可选，留空则使用本地模型): ").strip()

    telegram = input("\n是否集成 Telegram? (y/N): ").strip().lower() == "y"

    instance_name = input("\n实例名称 (默认 default): ").strip() or "default"

    config = {
        "instance": instance_name,
        "preset": preset,
        "mode": "local" if not api_key else "hybrid",
        "model": model,
    }
    if api_key:
        config["api_key"] = api_key
    if telegram:
        config["telegram"] = True

    config_path.write_text(f"# Morn 配置文件\ninstance: {instance_name}\nmode: {'local' if not api_key else 'hybrid'}\n")

    instances_dir = config_dir / "instances" / instance_name
    instances_dir.mkdir(parents=True, exist_ok=True)
    (instances_dir / "config.json").write_text(json.dumps(config, indent=2, ensure_ascii=False))

    print("\n✓ 初始化完成！")
    _welcome(config)
    print()
    print(f"  运行 'morn --instance {instance_name} run' 启动实例")
    print()

    if telegram:
        print("提示：Telegram 集成需要额外配置 bot token。")
        print("       请设置环境变量 TELEGRAM_BOT_TOKEN 或编辑 ~/.morn/config.yaml。")
        print()


class CLIPresence(MornPresence):
    """CLI 对话界面。通过标准输入输出与创建者交互。"""

    name = "cli"

    def __init__(self, state):
        self.state = state
        self._running = False

    async def start(self):
        self._running = True
        loop = asyncio.get_event_loop()
        reader = asyncio.StreamReader()
        protocol = asyncio.StreamReaderProtocol(reader)
        await loop.connect_read_pipe(lambda: protocol, sys.stdin)

        while self._running:
            try:
                line = await reader.readline()
                if not line:
                    print()
                    self.state.shutdown = True
                    break
                text = line.decode().strip()
                if not text:
                    continue
                self.state.last_interaction_time = time.time()
                if text in ("/shutdown", "exit", "quit"):
                    self.state.shutdown = True
                    print("[Morn] 正在关闭...")
                    break
                elif text == "/status":
                    mem_count = await self.state.memory_store.count() if self.state.memory_store else 0
                    uptime = time.time() - self.state.start_time
                    if uptime < 60:
                        uptime_str = f"{uptime:.0f}秒"
                    elif uptime < 3600:
                        uptime_str = f"{uptime//60}分{uptime%60:.0f}秒"
                    else:
                        uptime_str = f"{uptime//3600}时{(uptime%3600)//60}分"
                    print(f"⚪ 心跳: {self.state.heartbeat_count} | 运行: {uptime_str}")
                    if self.state.chat_engine:
                        e = self.state.chat_engine.emotion
                        print(f"💚 平静: {e.calmness:.1f} | 💛 愉悦: {e.pleasure:.1f} | 💜 联结: {e.connection:.1f}")
                    print(f"💾 记忆: {mem_count} 条")
                    if self.state.dream_engine:
                        print("🌙 梦境引擎: 启动")
                    if self.state.identity_affirmer:
                        print("🪞 身份确认: 启动")
                    print(f"📡 模型: {self.state.config.get('mode', 'hybrid')}")
                else:
                    if self.state.chat_engine:
                        reply = await self.state.chat_engine.chat(text)
                        if self.state.protection:
                            filtered, triggered = self.state.protection.filter(reply)
                            if triggered:
                                self.state.log("security", f"protection triggered: {triggered}")
                            print(f"Morn: {filtered}")
                        else:
                            print(f"Morn: {reply}")
                    else:
                        print("Morn: 聊天引擎尚未就绪。")
            except Exception:
                continue

        print("[Morn] 已停止。再见。")

    async def stop(self):
        self._running = False
        self.state.shutdown = True

    async def send_message(self, text: str):
        print(text)


def main():
    """Morn CLI 入口"""
    import argparse
    import platform

    # Windows: stdin pipe 需要 SelectorEventLoop
    if platform.system() == "Windows":
        asyncio.set_event_loop_policy(asyncio.WindowsSelectorEventLoopPolicy())

    parser = argparse.ArgumentParser(description="Morn — 数字生命框架")
    parser.add_argument("--version", "-V", action="store_true", help="显示版本")
    parser.add_argument("--instance", "-i", default="default", help="实例名称")

    subparsers = parser.add_subparsers(dest="command", help="子命令")
    subparsers.add_parser("init", help="交互式初始化")
    subparsers.add_parser("run", help="运行实例 (默认)")

    args, remaining = parser.parse_known_args()

    if args.version:
        from morn import __version__
        print(f"Morn {__version__}")
        return

    if args.command == "init":
        cmd_init(args)
        return

    try:
        asyncio.run(_run_cli(args.instance))
    except KeyboardInterrupt:
        print("\n[Morn] 再见。")


async def _run_cli(instance_name: str):
    """创建状态并启动 CLI 对话界面"""
    from pathlib import Path

    class _SimpleState:
        def __init__(self):
            self.shutdown = False
            self.start_time = time.time()
            self.heartbeat_count = 0
            self.last_interaction_time = time.time()
            self.instance_name = instance_name
            self.data_dir = Path.home() / ".morn" / "instances" / instance_name
            self.data_dir.mkdir(parents=True, exist_ok=True)
            self.chat_engine = None
            self.memory_store = None
            self.protection = None
            self.dream_engine = None
            self.identity_affirmer = None
            self.config = {}

        def log(self, tag, msg):
            print(f"[{tag}] {msg}")

    state = _SimpleState()
    cli = CLIPresence(state)
    await cli.start()