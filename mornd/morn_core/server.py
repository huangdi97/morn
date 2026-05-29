import argparse
import asyncio
import json
import logging
import os
import signal
import sys
import time
from dataclasses import dataclass, field
from logging.handlers import RotatingFileHandler
from pathlib import Path
from typing import Optional

import psutil

from morn_core.eventbus.bus import EventBus, Event, Priority
from morn_core.eventbus.hooks import HookManager, HookRegistration
from morn_core.eventbus.plugin_registry import register_all_plugin_hooks
from morn_core.memory.store import MemoryStore
from morn_core.chat.engine import ChatEngine, EmotionState
from morn_core.heartbeat import heartbeat_loop, memory_monitor, wal_checkpoint
from morn_core.security.user_protection import UserProtection
from morn_core.consciousness.self_reflection import SelfReflection
from morn_core.consciousness.dream_engine import DreamEngine
from morn_core.consciousness.self_reflection import IdentityAffirmer
from morn_core.consciousness.self_pruning import SelfPruner

try:
    from morn_core.presence.telegram_bot import TelegramBot
    _HAS_TELEGRAM = True
except ImportError:
    _HAS_TELEGRAM = False
    TelegramBot = None


class _MemHistory(list):
    maxlen = 1440

    def append(self, item):
        super().append(item)
        if len(self) > self.maxlen:
            self.pop(0)


@dataclass
class MornState:
    instance_name: str = "default"
    data_dir: Path = Path.home() / ".morn" / "instances" / "default"
    heartbeat_count: int = 0
    last_heartbeat: float = 0.0
    mem_history: list = field(default_factory=list)
    shutdown: bool = False
    db: Optional[object] = None
    start_time: float = field(default_factory=time.time)
    memory_count: int = 0
    memory_store: Optional[object] = None
    chat_engine: Optional[object] = None
    protection: Optional[object] = None
    telegram_bot: Optional[object] = None
    config: dict = field(default_factory=dict)
    dream_engine: Optional[object] = None
    identity_affirmer: Optional[object] = None
    self_pruner: Optional[object] = None
    last_interaction_time: float = field(default_factory=time.time)
    apz_store: Optional[object] = None
    bond_tracker: Optional[object] = None
    retrieval_engine: Optional[object] = None
    layered_retrieval: Optional[object] = None
    dynamic_permissions: Optional[object] = None
    external_boundary: Optional[object] = None
    intent_drift_detector: Optional[object] = None
    challenge_mode: Optional[object] = None
    audit_agent: Optional[object] = None

    def __post_init__(self):
        self.mem_history = _MemHistory(self.mem_history)

    def log(self, module: str, message: str):
        logging.getLogger("morn").info(f"{module} | {message}")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Morn 数字生命框架")
    parser.add_argument("--instance", default="default", help="实例名称")
    return parser.parse_args()


def setup_logging(data_dir: Path) -> logging.Logger:
    logs_dir = data_dir / "logs"
    logs_dir.mkdir(parents=True, exist_ok=True)

    logger = logging.getLogger("morn")
    logger.setLevel(logging.INFO)

    fmt = logging.Formatter(
        "%(asctime)s | %(name)s | %(message)s", datefmt="%Y-%m-%dT%H:%M:%S"
    )

    file_handler = RotatingFileHandler(
        logs_dir / "morn.log", maxBytes=10 * 1024 * 1024, backupCount=3
    )
    file_handler.setFormatter(fmt)

    stream_handler = logging.StreamHandler(sys.stdout)
    stream_handler.setFormatter(fmt)

    logger.addHandler(file_handler)
    logger.addHandler(stream_handler)

    return logger


def load_config(data_dir: Path) -> dict:
    default_config = dict(
        instance_name="default", instance_type="平衡型", mode="hybrid",
        model_provider="deepseek", model_name="deepseek-chat",
        ollama_model="qwen2.5:1.5b", ollama_host="http://localhost:11434",
        telegram_token="", memory_retention_days=30, temperature=0.7,
        birth_completed=False, reflection_light_interval=60,
        reflection_deep_interval=300, self_pruning_enabled=True,
        self_prune_interval=600, max_capsules=10000, max_skills=50,
        max_emotion_history=1000, bond_tracker_enabled=True,
        retrieval_engine_enabled=True, layered_retrieval_enabled=True,
        dynamic_permissions_enabled=True, external_boundary_enabled=True,
        intent_drift_detector_enabled=True, challenge_mode_enabled=True,
        audit_agent_enabled=True, llm_audit_enabled=True,
        llm_extract_enabled=True, thinking_evolution_enabled=True,
        hindsight_enabled=True, hindsight_threshold_days=30,
        hindsight_min_emotion=0.5, speech_recognition_enabled=False,
        speech_synthesis_enabled=False,
        risk_preference="yellow",
        risk_cooling_period=30,
        security_hot_reload=True,
        watchdog_enabled=True,
    )
    config_path = data_dir / "config.json"
    if not config_path.exists():
        data_dir.mkdir(parents=True, exist_ok=True)
        with open(config_path, "w") as f:
            json.dump(default_config, f, indent=2, ensure_ascii=False)
        return dict(default_config)
    try:
        with open(config_path) as f:
            cfg = json.load(f)
        for key, val in default_config.items():
            cfg.setdefault(key, val)
        return cfg
    except (json.JSONDecodeError, IOError):
        return dict(default_config)


async def _safe_init(state, name, fn):
    try:
        result = fn()
        if asyncio.iscoroutine(result):
            result = await result
        state.log(name, f"{name} initialized")
        return result
    except Exception as e:
        state.log(name, f"init failed: {e}")
        return None




async def cli_loop(state: MornState):
    loop = asyncio.get_event_loop()
    reader = asyncio.StreamReader()
    protocol = asyncio.StreamReaderProtocol(reader)
    await loop.connect_read_pipe(lambda: protocol, sys.stdin)

    while True:
        try:
            line = await reader.readline()
            if not line:
                print()
                state.shutdown = True
                break
            text = line.decode().strip()
            if not text:
                continue
            state.last_interaction_time = time.time()
            if text in ("/shutdown", "exit", "quit"):
                state.shutdown = True
                print("[Morn] 正在关闭...")
                break
            elif text == "/status":
                mem_count = await state.memory_store.count() if state.memory_store else 0
                uptime = time.time() - state.start_time
                if uptime < 60:
                    uptime_str = f"{uptime:.0f}秒"
                elif uptime < 3600:
                    uptime_str = f"{uptime//60}分{uptime%60:.0f}秒"
                else:
                    uptime_str = f"{uptime//3600}时{(uptime%3600)//60}分"
                print(f"⚪ 心跳: {state.heartbeat_count} | 运行: {uptime_str}")
                if state.chat_engine:
                    e = state.chat_engine.emotion
                    print(f"💚 平静: {e.calmness:.1f} | 💛 愉悦: {e.pleasure:.1f} | 💜 联结: {e.connection:.1f}")
                print(f"💾 记忆: {mem_count} 条")
                if state.dream_engine:
                    print(f"🌙 梦境引擎: 启动")
                if state.identity_affirmer:
                    print(f"🪞 身份确认: 启动")
                print(f"📡 模型: {state.config.get('mode', 'hybrid')}")
            else:
                if state.chat_engine:
                    reply = await state.chat_engine.chat(text)
                    if state.protection:
                        filtered, triggered = state.protection.filter(reply)
                        if triggered:
                            state.log("security", f"protection triggered: {triggered}")
                        print(f"Morn: {filtered}")
                    else:
                        print(f"Morn: {reply}")
                else:
                    print("Morn: 聊天引擎尚未就绪。")
        except Exception:
            continue

    print("[Morn] 已停止。再见。")



async def main():
    args = parse_args()
    instance_name = args.instance
    data_dir = Path.home() / ".morn" / "instances" / instance_name
    data_dir.mkdir(parents=True, exist_ok=True)

    state = MornState(
        instance_name=instance_name,
        data_dir=data_dir,
    )

    config = load_config(data_dir)
    state.config = config
    state.log("server", f"config loaded: mode={config['mode']}")

    logger = setup_logging(data_dir)
    state.log("server", f"starting instance [{instance_name}]...")

    async def init_subsystems(state):
        from morn.contrib.security_advanced.apz_store import APZStore as ApzStore
        from morn_core.evolution.skill_lifecycle import SkillStore
        from morn_core.memory.retrieval import RetrievalEngine, LayeredRetrievalEngine
        from morn.contrib.security_advanced.risk_guard import DynamicPermissions
        from morn_core.security.external_boundary import ExternalBoundary
        from morn.contrib.security_advanced.ethical_judgment import IntentDriftDetector
        from morn_core.consciousness.self_pruning import ChallengeMode
        from morn_core.memory.audit_agent import AuditAgent
        from morn_core.evolution.l0_tuner import ThinkingStyleEvolver
        from morn_core.consciousness.milestones import MilestoneTracker
        from morn_core.consciousness.hindsight import HindsightEngine
        store = await _safe_init(state, "memory", lambda: MemoryStore(data_dir, enable_encryption=True, event_bus=event_bus).__aenter__())
        if store:
            state.memory_store = store
            state.db = store.db
            state.memory_count = await store.count()

        state.apz_store = await _safe_init(state, "apz", lambda: ApzStore(data_dir))

        if state.memory_store:
            config["_config_path"] = str(data_dir / "config.json")
            state.chat_engine = await _safe_init(state, "chat", lambda: ChatEngine(
                instance_name=instance_name,
                memory_store=state.memory_store,
                config=config,
                apz_store=getattr(state, 'apz_store', None),
            ))

        state.protection = await _safe_init(state, "security", lambda: UserProtection(event_bus=event_bus))

        if state.memory_store and getattr(state, 'apz_store', None):
            state.dream_engine = await _safe_init(state, "dream", lambda: DreamEngine(
                memory_store=state.memory_store, apz_store=state.apz_store,
            ))

        skill_store = await _safe_init(state, "skill_store", lambda: SkillStore(data_dir / "skills.db").__aenter__())
        state.self_pruner = await _safe_init(state, "self_pruner", lambda: SelfPruner(
            memory_store=state.memory_store,
            skill_store=skill_store,
            instance_name=instance_name,
            max_capsules=config.get("max_capsules", 10000),
            max_skills=config.get("max_skills", 50),
            max_emotion_history=config.get("max_emotion_history", 1000),
            enabled=config.get("self_pruning_enabled", True),
        ))

        if config.get("bond_tracker_enabled", True):
            try:
                from morn_core.emotion.bond_tracker import BondTracker
                bond = BondTracker(config)
                bond.set_data_dir(data_dir)
                bond.load()
                state.bond_tracker = bond
                state.log("bond", f"bond tracker initialized: bond={bond.get_bond():.4f}")
            except Exception as e:
                state.log("bond", f"init failed: {e}")

        if config.get("retrieval_engine_enabled", True) and state.memory_store:
            state.retrieval_engine = await _safe_init(state, "retrieval", lambda: RetrievalEngine(state.memory_store))

        if config.get("layered_retrieval_enabled", True) and state.memory_store:
            state.layered_retrieval = await _safe_init(state, "layered_retrieval", lambda: LayeredRetrievalEngine(
                state.memory_store, mode=config.get("layered_retrieval_mode", "balanced"),
            ))

        if config.get("dynamic_permissions_enabled", True):
            state.dynamic_permissions = await _safe_init(state, "permissions", lambda: DynamicPermissions(data_dir))

        if config.get("external_boundary_enabled", True):
            state.external_boundary = await _safe_init(state, "boundary", lambda: ExternalBoundary(data_dir, event_bus=event_bus))

        if config.get("intent_drift_detector_enabled", True):
            state.intent_drift_detector = await _safe_init(state, "drift", lambda: IntentDriftDetector())

        if config.get("challenge_mode_enabled", True) and state.memory_store and state.bond_tracker:
            state.challenge_mode = await _safe_init(state, "challenge", lambda: ChallengeMode(
                memory_store=state.memory_store, bond_tracker=state.bond_tracker,
            ))

        if config.get("audit_agent_enabled", True) and state.memory_store:
            state.audit_agent = await _safe_init(state, "audit", lambda: AuditAgent(memory_store=state.memory_store))

        if config.get("thinking_evolution_enabled", True):
            state.thinking_evolver = await _safe_init(state, "thinking", lambda: ThinkingStyleEvolver(config={
                "enabled": True, "data_dir": str(data_dir),
            }))

        state.milestone_tracker = await _safe_init(state, "milestone", lambda: MilestoneTracker(
            data_dir=data_dir, memory_store=state.memory_store, chat_engine=state.chat_engine,
        ))

        if state.memory_store:
            state.identity_affirmer = await _safe_init(state, "identity", lambda: IdentityAffirmer(
                memory_store=state.memory_store, instance_name=instance_name,
                milestone_tracker=getattr(state, 'milestone_tracker', None),
            ))

        state.hindsight_engine = await _safe_init(state, "hindsight", lambda: HindsightEngine(
            memory_store=state.memory_store, chat_engine=state.chat_engine, config=config,
        ))
    loop = asyncio.get_event_loop()
    event_bus = EventBus(loop)
    await event_bus.start()
    hook_manager = HookManager(event_bus)

    from morn_core.security.security_validator import SecurityValidator, watch_config_reload
    security_validator = SecurityValidator(config, event_bus)
    security_validator.set_config_path(str(data_dir / "config.json"))
    await event_bus.publish(Event(
        type="security.config_reloaded",
        payload={"rules_loaded": len(security_validator._rules)},
        source="security_validator",
        priority=Priority.MEDIUM,
    ))
    tasks = [
        asyncio.create_task(
            watch_config_reload(security_validator),
            name="morn-sec-reload"
        )
    ]

    from morn_core.action.cli_executor import CLIExecutor
    state.cli_executor = CLIExecutor(config, validator=security_validator, event_bus=event_bus)

    await init_subsystems(state)

    register_all_plugin_hooks(event_bus, hook_manager, state)

    from morn_core.eventbus.health_monitor import HealthMonitor
    health = HealthMonitor(event_bus, hook_manager, state)
    health.register_hooks(hook_manager)

    try:
        import systemd.daemon
        systemd.daemon.notify("READY=1")
        async def _watchdog_ping():
            while True:
                await asyncio.sleep(10)
                systemd.daemon.notify("WATCHDOG=1")
        tasks.append(asyncio.create_task(_watchdog_ping(), name="morn-watchdog"))
    except ImportError:
        pass

    self_reflection = None
    if state.chat_engine and state.memory_store:
        self_reflection = SelfReflection(
            memory_store=state.memory_store,
            emotion_state=state.chat_engine.emotion,
            instance_name=instance_name,
            light_interval=config.get("reflection_light_interval", 60),
            deep_interval=config.get("reflection_deep_interval", 300),
        )

    if self_reflection:
        async def self_reflection_event_callback(event):
            await self_reflection.light_reflection()
        hook_manager.register(HookRegistration(
            plugin_id="self_reflection",
            event_type="heartbeat.minute",
            callback=self_reflection_event_callback,
            timeout=15.0,
        ))

    if state.self_pruner and self_reflection:
        state.self_pruner.emotion_history_ref = self_reflection._emotion_history

    from morn.cli import CLIPresence
    cli = CLIPresence(state)
    tasks += [
        asyncio.create_task(heartbeat_loop(state, event_bus), name="morn-heartbeat"),
        asyncio.create_task(memory_monitor(state), name="morn-memmon"),
        asyncio.create_task(wal_checkpoint(state), name="morn-wal"),
        asyncio.create_task(cli.start(), name="morn-cli"),
    ]

    if config.get("telegram_token") and _HAS_TELEGRAM:
        try:
            state.telegram_bot = TelegramBot(
                token=config["telegram_token"],
                chat_engine=state.chat_engine,
                memory_store=state.memory_store,
                data_dir=data_dir,
                instance_name=instance_name,
            )
            tasks.append(
                asyncio.create_task(state.telegram_bot.start(), name="morn-telegram")
            )
            state.log("telegram", "bot started")
        except Exception as e:
            state.log("telegram", f"init failed: {e}")



    print(f"\nMorn v0.1 [{instance_name}] 已启动")
    print(f"  PID: {os.getpid()}")
    print(f"  数据目录: {data_dir}")
    print(f"  记忆: {state.memory_count if state.memory_store else 0} 条")
    if state.chat_engine:
        print(f"  情感: {state.chat_engine.emotion}")
    if state.dream_engine:
        print(f"  梦境引擎: 启动")
    if state.identity_affirmer:
        print(f"  身份确认: 启动")
    print(f"  Telegram: {'已连接' if state.telegram_bot else '未配置（仅命令行模式）'}")
    print(f"  模型: {config.get('mode', 'hybrid')}")
    print(f"\n输入 /status 查看状态，/shutdown 退出\n")

    await asyncio.gather(*tasks, return_exceptions=True)
    await event_bus.stop()

    if state.telegram_bot:
        await state.telegram_bot.stop()
    if state.memory_store:
        await state.memory_store.close()

    if state.heartbeat_count > 0:
        print(f"[Morn] 本次运行: {state.heartbeat_count} 次心跳 | 再见。")

    def handle_signal(signum, frame):
        print("\n[Morn] 正在关闭...")
        state.shutdown = True

    signal.signal(signal.SIGINT, handle_signal)
    signal.signal(signal.SIGTERM, handle_signal)


if __name__ == "__main__":
    asyncio.run(main())