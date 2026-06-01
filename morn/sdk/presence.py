"""Morn Presence 基类——对话界面接入点"""
from abc import ABC, abstractmethod


class MornPresence(ABC):
    """存在形式基类。所有对话界面通过此类接入 Morn 实例。"""

    name: str = "base"

    @abstractmethod
    async def start(self) -> None: ...

    @abstractmethod
    async def stop(self) -> None: ...

    @abstractmethod
    async def send_message(self, text: str) -> None: ...