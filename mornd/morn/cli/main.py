"""Morn CLI Presence — 默认对话界面"""
import asyncio
import sys
import time
from typing import Optional

from morn.sdk.presence import MornPresence


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
                        print(f"🌙 梦境引擎: 启动")
                    if self.state.identity_affirmer:
                        print(f"🪞 身份确认: 启动")
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