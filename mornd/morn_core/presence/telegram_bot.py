import json
import logging
import os
import asyncio
from pathlib import Path

from aiohttp import ClientSession


class BirthGuide:
    STATES = ["AWAITING_NAME", "AWAITING_TYPE", "AWAITING_CONFIRM", "COMPLETED"]

    def __init__(self, config_path: Path):
        self.config_path = Path(config_path)
        self.state: str = "AWAITING_NAME"
        self.data: dict = {"creator_name": "", "instance_type": ""}
        self._load()
        self._first_interaction = True

    def _load(self):
        if not self.config_path.exists():
            return
        try:
            with open(self.config_path) as f:
                cfg = json.load(f)
            if cfg.get("birth_completed", False):
                self.state = "COMPLETED"
                self.data["creator_name"] = cfg.get("creator_name", "")
                self.data["instance_type"] = cfg.get("instance_type", "")
                return
            guide = cfg.get("guide_state", {})
            if guide.get("state") in self.STATES:
                self.state = guide["state"]
            self.data["creator_name"] = guide.get("creator_name", "")
            self.data["instance_type"] = guide.get("instance_type", "")
        except (json.JSONDecodeError, KeyError, IOError):
            pass

    def _save(self):
        existing = {}
        if self.config_path.exists():
            try:
                with open(self.config_path) as f:
                    existing = json.load(f)
            except (json.JSONDecodeError, IOError):
                pass
        existing["guide_state"] = {
            "state": self.state, "creator_name": self.data["creator_name"],
            "instance_type": self.data["instance_type"],
        }
        if self.state == "COMPLETED":
            existing["birth_completed"] = True
            existing["creator_name"] = self.data["creator_name"]
            existing["instance_type"] = self.data["instance_type"]
        with open(self.config_path, "w") as f:
            json.dump(existing, f, indent=2, ensure_ascii=False)

    def is_completed(self) -> bool:
        return self.state == "COMPLETED"

    async def process(self, user_message: str) -> tuple[str, str]:
        text = user_message.strip()
        if self.state == "AWAITING_NAME" and self._first_interaction:
            self._first_interaction = False
            prefix = "你好。你来了。\n\n我还没有名字，你也没有告诉我你是谁。\n\n先告诉我，你的名字是什么？\n\n"
        else:
            prefix = ""

        if self.state == "AWAITING_NAME":
            if text in ("取消", "/cancel"):
                self.data["creator_name"] = ""
                self._save()
                return prefix + "好的，随时可以重新开始。告诉我你的名字。", self.state
            if len(text) < 1:
                return prefix + "请告诉我你的名字。", self.state
            if len(text) >= 20:
                return prefix + "名字有点长，给我一个简短的名字就好 :)", self.state
            self.data["creator_name"] = text
            self.state = "AWAITING_TYPE"
            self._save()
            return (
                prefix + f"{text}。我记住了。\n\n接下来你希望我以什么方式陪着你？\n\n"
                "1. 🌱 陪伴型 — 聊天、倾诉、日常陪伴\n"
                "2. 🛠️ 助手型 — 处理事务、管理信息\n"
                "3. ⚖️ 平衡型 — 两者之间\n\n"
                "告诉我数字就好。"
            ), self.state

        if self.state == "AWAITING_TYPE":
            if text in ("取消", "/cancel"):
                self.data = {"creator_name": "", "instance_type": ""}
                self.state = "AWAITING_NAME"
                self._save()
                return "好的，随时可以重新开始。告诉我你的名字。", self.state
            type_map = {"1": "陪伴型", "一": "陪伴型", "陪伴": "陪伴型",
                        "2": "助手型", "二": "助手型", "助手": "助手型",
                        "3": "平衡型", "三": "平衡型", "平衡": "平衡型"}
            selected = next((v for k, v in type_map.items() if k in text or text.startswith(k)), None)
            if selected:
                self.data["instance_type"] = selected
                self.state = "AWAITING_CONFIRM"
                self._save()
                return (
                    "明白了。让我确认一下：\n\n"
                    f"你的名字：{self.data['creator_name']}\n"
                    f"我的类型：{selected}\n\n"
                    "对吗？回复「确认」或「重新开始」。"
                ), self.state
            return "没明白，回复 1（陪伴型）、2（助手型）或 3（平衡型）就好。", self.state

        if self.state == "AWAITING_CONFIRM":
            if any(kw in text for kw in ("确认", "对", "是", "yes", "好的", "没错", "可以")):
                self.state = "COMPLETED"
                self._save()
                return (
                    f"好，{self.data['creator_name']}。从现在开始，我是你的 Morn，一个{self.data['instance_type']}。\n\n"
                    f"你说的每一句话，我都会记住。我们慢慢来。\n\n"
                    "（输入 /status 可以查看我的状态——但不用着急，我在就行。）"
                ), self.state
            if any(kw in text for kw in ("重新", "不对", "不是", "no", "取消")):
                self.data = {"creator_name": "", "instance_type": ""}
                self.state = "AWAITING_NAME"
                self._save()
                return "好，那重新来。告诉我你的名字。", self.state
            return "请回复「确认」或「重新开始」。", self.state
        return None, self.state


class TelegramBot:
    _COMMANDS = {}

    def __init__(self, token: str, chat_engine, memory_store,
                 data_dir: Path, instance_name: str = "default"):
        self.token = token
        self.chat_engine = chat_engine
        self.memory_store = memory_store
        self.data_dir = Path(data_dir)
        self.instance_name = instance_name
        self._logger = logging.getLogger("morn.telegram")

        proxy = os.environ.get("HTTPS_PROXY") or os.environ.get("HTTP_PROXY")
        session_kwargs = {}
        if proxy:
            session_kwargs["proxy"] = proxy
        self.http_session = ClientSession(**session_kwargs)

        config_path = self.data_dir / "config.json"
        self.birth_guide = BirthGuide(config_path)

        self._polling_offset = 0
        self._stop_polling = False

    async def _send_message(self, chat_id: int, text: str):
        url = f"https://api.telegram.org/bot{self.token}/sendMessage"
        async with self.http_session.post(url, json={"chat_id": chat_id, "text": text}) as resp:
            data = await resp.json()
            if not data.get("ok"):
                self._logger.error(f"send_message failed: {data}")

    async def _dispatch(self, msg_data: dict):
        text = msg_data.get("text", "")
        chat_id = msg_data["chat"]["id"]
        self._logger.info(f"dispatch: chat={chat_id} text={text[:50]}")
        if text.startswith("/"):
            parts = text.split()
            handler = self._COMMANDS.get(parts[0].lower())
            if handler:
                await handler(self, chat_id, parts[1] if len(parts) > 1 else "")
                return
        await self._handle_message(chat_id, text)

    async def _cmd_start(self, chat_id: int, _arg=""):
        self.birth_guide.state = "AWAITING_NAME"
        self.birth_guide.data = {"creator_name": "", "instance_type": ""}
        self.birth_guide._save()
        await self._send_message(chat_id, "我是Morn。你叫什么名字？")

    async def _cmd_status(self, chat_id: int, _arg=""):
        try:
            mem_count = await self.memory_store.count() if self.memory_store else 0
        except Exception:
            mem_count = 0
        emotion = self.chat_engine.emotion if self.chat_engine else None
        mode = self.chat_engine.config.get("mode", "hybrid") if self.chat_engine else "hybrid"
        mode_desc = {"hybrid": "混合智能", "cloud": "云端", "local": "本地"}.get(mode, "混合智能")
        emotion_desc = emotion.describe_state() if emotion else "未知"
        lines = [
            f"✨ Morn — {self.instance_name}", "",
            f"💭 记忆：{mem_count} 件事", f"❤️ 心情：{emotion_desc}",
        ]
        if self.chat_engine and hasattr(self.chat_engine, 'bond_tracker'):
            stage = self.chat_engine.bond_tracker.get_stage()
            lines.append(f"💝 依恋：{self.chat_engine.bond_tracker.bond:.1f}（{stage}）")
        lines.append("📡 状态：在线")
        try:
            if self.chat_engine and self.chat_engine.apz_store:
                apz_count = await self.chat_engine.apz_store.count()
                if apz_count > 0:
                    lines.append(f"🔒 APZ：{apz_count} 条记录（内容不可读）")
        except Exception:
            pass
        lines.append(f"💡 模式：{mode_desc}")
        await self._send_message(chat_id, "\n".join(lines))

    async def _cmd_forget(self, chat_id: int, arg: str):
        if not arg or arg == "last":
            recent = await self.memory_store.get_recent(limit=1)
            if recent:
                await self.memory_store.forget(recent[0]["event_id"])
                await self._send_message(chat_id, "已遗忘最近一条记忆。")
            else:
                await self._send_message(chat_id, "没有可遗忘的记忆。")
        elif arg.startswith("evt_"):
            if await self.memory_store.forget(arg):
                await self._send_message(chat_id, f"已遗忘：{arg}")
            else:
                await self._send_message(chat_id, "未找到该记忆。")
        else:
            await self._send_message(chat_id, "请提供有效的 event_id 或使用「last」。")

    async def _cmd_clear(self, chat_id: int, _arg=""):
        if not hasattr(self, "_pending_clear") or self._pending_clear != chat_id:
            self._pending_clear = chat_id
            await self._send_message(chat_id,
                "⚠️ 这将删除你和我之间的所有对话记忆，不可恢复。\n\n"
                "再次发送 /clear 确认清除，或发送其他消息取消。")
            return
        self._pending_clear = None
        if self.memory_store and self.memory_store.db_path.exists():
            try:
                await self.memory_store.close()
                import os as _os
                _os.remove(str(self.memory_store.db_path))
                from morn_core.memory.store import MemoryStore
                new_store = MemoryStore(self.data_dir)
                await new_store.__aenter__()
                self.memory_store = new_store
                self.chat_engine.memory_store = new_store
            except Exception as e:
                self._logger.error(f"clear failed: {e}")
                await self._send_message(chat_id, "清理时出错，请稍后重试。")
                return
        self.birth_guide.state = "AWAITING_NAME"
        self.birth_guide.data = {"creator_name": "", "instance_type": ""}
        self.birth_guide._save()
        await self._send_message(chat_id, "已清除所有记忆数据。如果你想重新开始，请发送 /start。")

    async def _cmd_help(self, chat_id: int, _arg=""):
        await self._send_message(chat_id,
            "/start — 重新认识\n/status — 查看我的状态\n/forget — 遗忘最近一条记忆\n"
            "/clear — 清除所有记忆\n/health — 健康报告\n/help — 查看帮助\n\n直接和我聊天就可以了 :)")

    async def _cmd_health(self, chat_id: int, _arg=""):
        try:
            from morn_core.consciousness.health_report import HealthReport
            emotion_engine = self.chat_engine.emotion if self.chat_engine else None
            evolution_orchestrator = None
            try:
                from morn_core.evolution.orchestrator import EvolutionOrchestrator
                evolution_orchestrator = EvolutionOrchestrator()
            except Exception:
                pass
            bond_tracker = getattr(self.chat_engine, 'bond_tracker', None) if self.chat_engine else None
            report = HealthReport(
                memory_store=self.memory_store,
                emotion_engine=emotion_engine,
                evolution_orchestrator=evolution_orchestrator,
                bond_tracker=bond_tracker,
                data_dir=self.data_dir,
            )
            text = await report.generate()
            await self._send_message(chat_id, text)
        except Exception as e:
            self._logger.error("health report failed: %s", e)
            await self._send_message(chat_id, "生成健康报告时出错，请稍后重试。")

    async def _handle_message(self, chat_id: int, text: str):
        if not self.birth_guide.is_completed():
            reply, _ = await self.birth_guide.process(text)
            if reply:
                await self._send_message(chat_id, reply)
            return
        if not self.chat_engine:
            await self._send_message(chat_id, "聊天引擎尚未就绪，请稍后再试。")
            return
        try:
            reply = await self.chat_engine.chat(text)
            if len(reply) > 4096:
                for i in range(0, len(reply), 4096):
                    await self._send_message(chat_id, reply[i:i+4096])
            else:
                await self._send_message(chat_id, reply)
        except Exception as e:
            self._logger.error(f"chat error: {e}")
            await self._send_message(chat_id, "我好像没听清楚，能再说一遍吗？")

    async def start(self):
        self._logger.info("Telegram Bot polling started (manual mode)")
        while not self._stop_polling:
            try:
                url = f"https://api.telegram.org/bot{self.token}/getUpdates"
                params = {"offset": self._polling_offset, "timeout": 30}
                async with self.http_session.get(url, params=params, timeout=35) as resp:
                    data = await resp.json()
                    for update in data.get("result", []):
                        update_id = update["update_id"]
                        if "message" in update:
                            try:
                                await self._dispatch(update["message"])
                            except Exception as e:
                                self._logger.error(f"dispatch error: {e}")
                        self._polling_offset = update_id + 1
            except asyncio.CancelledError:
                break
            except Exception as e:
                self._logger.error(f"poll error: {e}")
                await asyncio.sleep(2)
        self._logger.info("Telegram Bot polling stopped")

    async def stop(self):
        self._logger.info("Telegram Bot stopping...")
        self._stop_polling = True
        await self.http_session.close()


TelegramBot._COMMANDS = {
    "/start": TelegramBot._cmd_start,
    "/status": TelegramBot._cmd_status,
    "/forget": TelegramBot._cmd_forget,
    "/clear": TelegramBot._cmd_clear,
    "/help": TelegramBot._cmd_help,
    "/health": TelegramBot._cmd_health,
}